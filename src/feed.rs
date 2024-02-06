use chrono::{DateTime, Duration, Utc};
use feed_rs::parser;
use std::error::Error;
use std::io::{self, Write};
use ureq::get;

const FEEDS: &str = r#"https://asahilinux.org/blog/index.xml
https://blog.rust-lang.org/feed.xml
https://drewdevault.com/blog/index.xml
https://endtimes.dev/feed.xml
https://github.com/hellux/jotdown/tags.atom
https://hllmn.net/blog/index.xml
https://kmaasrud.com/atom.xml
https://nutcroft.com/rss/"#;

pub fn mock_feeds() -> Result<Feeds, Box<dyn Error>> {
    let mut feeds = Feeds::new();

    for url in FEEDS.split('\n') {
        let mut bytes: Vec<u8> = Vec::new();
        get(url).call()?.into_reader().read_to_end(&mut bytes)?;

        feeds.push(Feed::try_from(
            parser::parse::<&[u8]>(bytes.as_ref()).map_err(|e| e.to_string())?,
        )?);
    }

    Ok(feeds)
}

fn format_duration(dur: Duration) -> String {
    let mut val = dur.num_minutes();
    let mut descriptor = "minute";

    if dur.num_weeks() > 52 {
        descriptor = "year";
        val = dur.num_days() / 365;
    } else if dur.num_weeks() > 4 {
        descriptor = "month";
        val = dur.num_weeks() / 4;
    } else if dur.num_days() > 7 {
        descriptor = "week";
        val = dur.num_weeks();
    } else if dur.num_hours() > 24 {
        descriptor = "day";
        val = dur.num_days();
    } else if dur.num_minutes() > 60 {
        descriptor = "hour";
        val = dur.num_hours();
    }

    let mut s = format!("{val} {descriptor}");
    if val > 1 {
        s.push('s');
    }
    s
}

#[derive(Clone)]
pub struct Entry {
    title: String,
    link: Option<String>,
    dt: Option<DateTime<Utc>>,
}

impl Entry {
    pub fn domain(&self) -> Option<String> {
        // FIXME: this is very naïve URL parsing
        self.link
            .as_ref()
            .map(|link| link.split('/').skip(2).take(1).collect())
    }

    pub fn duration_since(&self) -> String {
        self.dt
            .map(|dt| Utc::now() - dt)
            .map(|dur| format_duration(dur).to_string())
            .unwrap_or("some time".to_string())
    }

    pub fn write_html<W: Write>(&self, mut w: W) -> io::Result<()> {
        write!(w, "<a")?;
        if let Some(ref link) = self.link {
            write!(w, " href=\"{link}\"")?;
        }
        write!(w, ">{}</a><br>", &self.title)?;

        write!(
            w,
            "<span class=\"entry-sub\">published {} ago",
            self.duration_since()
        )?;
        if let Some(domain) = self.domain() {
            // TODO: we probably want a link to the blog root, if available
            write!(w, " on {domain}")?;
        }
        write!(w, "</span>")
    }
}

impl TryFrom<feed_rs::model::Entry> for Entry {
    type Error = Box<dyn Error>;

    fn try_from(value: feed_rs::model::Entry) -> Result<Self, Self::Error> {
        Ok(Entry {
            title: value
                .title
                .ok_or_else(|| format!("No title for feed entry {}", value.id))?
                .content,
            link: value.links.first().map(|l| l.href.to_owned()),
            dt: value.published.or(value.updated),
        })
    }
}

pub struct Feed {
    entries: Vec<Entry>,
}

impl TryFrom<feed_rs::model::Feed> for Feed {
    type Error = Box<dyn Error>;

    fn try_from(value: feed_rs::model::Feed) -> Result<Self, Self::Error> {
        Ok(Feed {
            entries: value
                .entries
                .iter()
                .map(|entry| Entry::try_from(entry.clone()))
                .collect::<Result<Vec<Entry>, Self::Error>>()?,
        })
    }
}

pub struct Feeds {
    feeds: Vec<Feed>,
}

impl Feeds {
    fn new() -> Self {
        Self { feeds: Vec::new() }
    }

    fn push(&mut self, feed: Feed) {
        self.feeds.push(feed)
    }

    fn entries_sorted(&self) -> Vec<Entry> {
        let mut entries: Vec<Entry> = self
            .feeds
            .iter()
            .flat_map(|feed| feed.entries.clone())
            .collect();

        entries.sort_unstable_by(|e1, e2| e2.dt.cmp(&e1.dt));
        entries
    }

    pub fn write_html<W: Write>(&self, mut w: W) -> io::Result<()> {
        write!(w, "<ul>")?;
        for entry in &self.entries_sorted() {
            write!(w, "<li>")?;
            entry.write_html(&mut w)?;
            write!(w, "</li>")?;
        }
        write!(w, "</ul>")?;
        Ok(())
    }
}

use crate::time::format_duration;

use chrono::{DateTime, Utc};
use feed_rs::parser;
use opml::OPML;
use std::error::Error;
use std::io::{self, Write};
use std::time::Duration;
use ureq::get;

const TIMEOUT: Duration = Duration::new(10, 0);

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
    pub fn new() -> Self {
        Self { feeds: Vec::new() }
    }

    fn push(&mut self, feed: Feed) {
        self.feeds.push(feed)
    }

    pub fn push_from_opml(&mut self, opml_url: &str) -> Result<(), Box<dyn Error>> {
        let mut r = get(opml_url).timeout(TIMEOUT).call()?.into_reader();
        let opml = OPML::from_reader(&mut r)?;

        for outline in opml.body.outlines {
            if let Some(url) = outline.xml_url {
                match get(&url).timeout(TIMEOUT).call() {
                    Ok(response) => {
                        let mut r = response.into_reader();
                        match parser::parse(&mut r) {
                            Ok(parsed) => self.push(Feed::try_from(parsed)?),
                            Err(e) => eprintln!("error: failed to parse feed from {url}: {e}"),
                        }
                    }
                    Err(e) => eprintln!("error: failed when fetching feed: {e}"),
                }
            }
        }

        Ok(())
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

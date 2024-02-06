mod feed;

use std::error::Error;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use feed::Feeds;

const ADDR: &str = "0.0.0.0:8080";
const UPDATE_MINUTES: u64 = 15;
const BUF_SIZE: usize = 1024;
const OPML_URLS: [&str; 1] = ["https://kmaasrud.com/blogroll.xml"];

struct Model {
    opml_urls: [&'static str; 1],
    feeds: Feeds,
}

impl Model {
    fn new() -> Self {
        Self {
            opml_urls: OPML_URLS,
            feeds: Feeds::new(),
        }
    }

    fn update(&mut self) -> Result<(), Box<dyn Error>> {
        self.feeds = Feeds::new();
        for url in self.opml_urls {
            self.feeds.push_from_opml(url)?;
        }
        Ok(())
    }

    fn view<W: Write>(&self, mut w: W) -> Result<(), Box<dyn Error>> {
        write!(w, "<!DOCTYPE html>")?;
        write!(w, r#"<meta charset="utf-8">"#)?;
        write!(
            w,
            r#"<meta name="viewport" content="width=device-width, initial-scale=1">"#
        )?;
        write!(w, r#"<meta name="color-scheme" content="light dark">"#)?;
        write!(
            w,
            r#"<link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>📰</text></svg>">"#
        )?;
        write!(w, "<style>{}</style>", include_str!("./main.css"))?;
        write!(w, "<h1>fdrd <sup>the tiny feed reader</sup></h1>")?;
        self.feeds.write_html(&mut w)?;
        Ok(())
    }
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut model = Model::new();
    model.update()?;
    eprintln!("performed initial model update");

    let model = Arc::new(Mutex::new(model));
    let model_clone = Arc::clone(&model);
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(UPDATE_MINUTES * 60));
        match model_clone.try_lock() {
            Ok(mut feeds) => {
                if let Err(e) = feeds.update() {
                    eprintln!("error: failed to update model: {e}");
                }
                eprintln!("model updated!");
            }
            Err(e) => eprintln!("error: failed to acquire lock when updating model: {e}"),
        }
    });

    let listener = TcpListener::bind(ADDR)?;
    eprintln!("server listening on {ADDR}...");

    for stream in listener.incoming() {
        let stream = stream?;
        let model = Arc::clone(&model);
        thread::spawn(move || {
            if let Err(e) = handle_client(stream, model) {
                eprintln!("error: failed to handle client: {e}");
            }
        });
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream, model: Arc<Mutex<Model>>) -> Result<(), Box<dyn Error>> {
    eprintln!(
        "connected to: {}",
        stream
            .peer_addr()
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default()
    );

    // NOTE: We only need to check the request line, so this should be enough of a buffer
    let mut buffer = [0; BUF_SIZE];
    match stream.read(&mut buffer) {
        Ok(_) => {
            let model = match model.try_lock() {
                Ok(model) => model,
                Err(e) => {
                    writeln!(stream, "HTTP/1.1 500 Internal Server Error\r")?;
                    writeln!(stream, "Content-Type: text/plain\r\n\r")?;
                    writeln!(stream, "500 Internal Server Error: {e}")?;
                    return Err(e.to_string().into());
                }
            };

            if is_get_root(buffer) {
                writeln!(stream, "HTTP/1.1 200 OK\r").unwrap();
                writeln!(stream, "Content-Type: text/html; charset=UTF-8\r\n\r")?;
                model.view(&mut stream)?;
            } else {
                write!(stream, "HTTP/1.1 404 NOT FOUND\r\n\r\n404 Page not found")?;
            }
        }
        Err(e) => eprintln!("error: failed to read from socket: {}", e),
    }
    Ok(())
}

fn is_get_root<const N: usize>(request: [u8; N]) -> bool {
    std::str::from_utf8(&request)
        .ok()
        .unwrap_or_default()
        .lines()
        .next()
        .map(|first_line| {
            first_line.starts_with("GET / ") || first_line.starts_with("GET /index.html ")
        })
        .unwrap_or(false)
}

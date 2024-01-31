use std::error::Error;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

const ADDR: &str = "192.168.0.60:8080";

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
    let listener = TcpListener::bind(ADDR)?;

    eprintln!("server listening on {ADDR}...");

    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(|| {
            handle_client(stream);
        });
    }

    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    eprintln!(
        "connected to: {}",
        stream
            .peer_addr()
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default()
    );

    // NOTE: We only need to check the request line, so this should be enough of a buffer
    let mut buffer = [0; 1024];
    match stream.read(&mut buffer) {
        Ok(_) => {
            let request = std::str::from_utf8(&buffer).unwrap();

            if is_get_root(request) {
                writeln!(stream, "HTTP/1.1 200 OK\r").unwrap();
                writeln!(stream, "Content-Type: text/html; charset=UTF-8\r\n\r").unwrap();
                write_main_page(&mut stream).unwrap();
            } else {
                write!(stream, "HTTP/1.1 404 NOT FOUND\r\n\r\n404 Page not found").unwrap();
            }

            stream.flush().unwrap();
        }
        Err(e) => eprintln!("error: failed to read from socket: {}", e),
    }
}

fn is_get_root(request: &str) -> bool {
    request
        .lines()
        .next()
        .map(|first_line| {
            first_line.starts_with("GET / ") || first_line.starts_with("GET /index.html ")
        })
        .unwrap_or(false)
}

fn write_main_page<W: Write>(mut w: W) -> io::Result<()> {
    write!(w, "<!DOCTYPE html>")?;
    write!(w, r#"<meta charset="utf-8">"#)?;
    write!(
        w,
        r#"<meta name="viewport" content="width=device-width, initial-scale=1">"#
    )?;
    write!(w, r#"<meta name="color-scheme" content="light dark">"#)?;
    write!(
        w,
        r#"<link rel="icon" href="data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>💭</text></svg>">"#
    )?;
    write!(w, "<h1>Welcome to this RSS reader!</h1>")?;
    write!(w, "<p>Isn't this fun?</p>")?;
    Ok(())
}

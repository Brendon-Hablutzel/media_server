use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    str, thread,
};
use types::{HttpResponseBytes, Errors, Request};
mod pages;
mod types;
mod utils;

fn route(request: Request, media_dir: &str) -> Result<HttpResponseBytes, Errors> {
    if request.get_method() != "GET" {
        return Err(Errors::InvalidMethod);
    }

    match request.get_endpoint() {
        "/" => pages::index(media_dir),
        route => pages::media(&request, route, media_dir),
    }
}

fn handle_connection(mut stream: TcpStream, media_dir: &str) {
    let buffer_reader = BufReader::new(&mut stream);

    let response = Request::from_buffer(buffer_reader)
        .map_err(|err| Errors::ClientError(format!("Error parsing request: {err}")))
        .and_then(|request| {
            println!("REQUEST:");
            println!("{request}");
            println!("----");

            route(request, media_dir)
        })
        .unwrap_or_else(|err| err.get_page());

    stream
        .write(&response)
        .expect("Should be able to write to TCP stream");
}

fn main() -> Result<(), String> {
    let mut args = std::env::args();
    args.next(); // skip first argument

    let port = args.next().ok_or("Port is a required argument")?;

    let media_dir = utils::get_env("MEDIA_DIR")?;

    let server = TcpListener::bind(format!("0.0.0.0:{port}"))
        .map_err(|err| format!("Unable to create TCP listener: {err}"))?;

    println!(
        "Started server at {}...",
        server
            .local_addr()
            .map_err(|err| { format!("Unable to get TCP listener address: {err}") })?
    );

    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                let media_dir = media_dir.clone();
                thread::spawn(move || {
                    handle_connection(stream, &media_dir);
                });
            }
            Err(e) => eprintln!("Error establishing connection: {e}"),
        }
    }

    Ok(())
}

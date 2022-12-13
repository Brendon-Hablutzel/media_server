use crate::{pages, utils};
use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead, BufReader},
    net::TcpStream,
};

#[derive(Debug)]
pub enum Errors {
    NotFound,
    ClientError(String),
    ServerError(String),
    InvalidContentRange,
    InvalidMethod,
}

impl Errors {
    pub fn get_page(&self) -> HttpResponseBytes {
        let error_code = match self {
            Errors::NotFound => 404,
            Errors::ServerError(e) => {
                eprintln!("Error generating response: {e}");
                500
            }
            Errors::ClientError(e) => {
                eprintln!("Error parsing request: {e}");
                400
            }
            Errors::InvalidContentRange => 416,
            Errors::InvalidMethod => 405,
        };
        pages::error(error_code)
    }
}

pub type ByteRange = (u64, Option<u64>);

pub type HttpResponseBytes = Vec<u8>;

pub struct Request {
    method: String,
    endpoint: String,
    headers: HashMap<String, String>,
    range: Option<ByteRange>,
}

impl Request {
    pub fn from_buffer(buffer_reader: BufReader<&mut TcpStream>) -> Result<Self, String> {
        let mut lines = buffer_reader.lines();

        let start_line = lines
            .next()
            .ok_or("Start line empty")?
            .map_err(|err| format!("Error parsing request start line: {err}"))?;

        let mut start_line = start_line.split(" ");

        let method = start_line
            .next()
            .ok_or("No method found in request")?
            .to_owned();

        let endpoint = start_line
            .next()
            .ok_or("No endpoint found in request")?
            .to_owned();

        let mut range: Option<ByteRange> = None;

        let mut headers = HashMap::new();
        loop {
            if let Some(line) = lines.next() {
                let line =
                    line.map_err(|err| format!("Error getting next line of request: {err}"))?;

                if line == "" {
                    break;
                } else {
                    let (name, value) = line.split_once(": ").ok_or("Error parsing header")?;
                    headers.insert(name.to_owned(), value.to_owned());

                    if name == "Range" {
                        range = utils::parse_range_header(&line)?;
                    }
                }
            } else {
                break;
            }
        }

        Ok(Request {
            method,
            endpoint,
            headers,
            range,
        })
    }

    pub fn get_method(&self) -> &str {
        &self.method
    }

    pub fn get_endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn get_range(&self) -> Option<ByteRange> {
        self.range
    }
}

impl Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let headers = self
            .headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}\n"))
            .collect::<String>();

        write!(
            f,
            "{} {}\n{}",
            self.method,
            self.endpoint,
            headers.trim_end()
        )
    }
}

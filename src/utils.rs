use crate::types::{ByteRange, Errors};
use std::{
    env::{self, VarError},
    fs,
    io::ErrorKind,
};

pub fn get_file(media_dir: &str, desired_filename: &str) -> Result<Vec<u8>, Errors> {
    let available_files = list_available_files(media_dir)?;

    if !available_files.contains(&desired_filename.to_owned()) {
        return Err(Errors::NotFound);
    };

    let file_path = format!("{media_dir}/{desired_filename}");

    fs::read(file_path).map_err(|err| match err.kind() {
        ErrorKind::NotFound => Errors::NotFound, // unlikely to happen but should be covered anyways
        _ => Errors::ServerError(err.to_string()),
    })
}

pub fn get_content_type(file_name: &str) -> Result<String, String> {
    let extension = match file_name.split_once(".") {
        Some((_name, extension)) => Ok(extension),
        None => Err("Unable to get file type".to_owned()),
    }?;

    match extension {
        "mp3" => Ok("audio/mpeg"),
        "csv" | "txt" => Ok("text/plain"),
        "jpg" | "jpeg" => Ok("image/jpeg"),
        "png" => Ok("image/png"),
        _ => Err("Invalid file extension".to_owned()),
    }
    .map(|file_type| file_type.to_owned())
}

pub fn list_available_files(media_dir: &str) -> Result<Vec<String>, Errors> {
    let files = fs::read_dir(media_dir)
        .map_err(|err| Errors::ServerError(err.to_string()))?
        .filter_map(|entry| {
            let entry_path = entry.ok()?.path();

            if !entry_path.is_file() {
                return None;
            }

            entry_path
                .file_name()
                .and_then(|os_str| os_str.to_str())
                .map(|file_name| file_name.to_owned())
        })
        .collect();

    Ok(files)
}

pub fn parse_range_header(line: &str) -> Result<Option<ByteRange>, String> {
    let (_, byte_range) = line.split_once("=").ok_or("Error parsing Range header")?;

    let (start, end) = byte_range
        .split_once("-")
        .ok_or("Error parsing Range header")?;

    let start: u64 = start
        .parse()
        .map_err(|err| format!("Error parsing Range byte range: {err}"))?;

    let end: Option<u64> = match end {
        "" => None,
        end => Some(
            end.parse()
                .map_err(|err| format!("Error parsing Range byte range: {err}"))?,
        ),
    };

    Ok(Some((start, end)))
}

pub fn get_env(name: &str) -> Result<String, String> {
    env::var(name).map_err(|err| match err {
        VarError::NotPresent => format!("Unable to find env variable {name}"),
        VarError::NotUnicode(_) => format!("Unable to parse env variable {name}"),
    })
}

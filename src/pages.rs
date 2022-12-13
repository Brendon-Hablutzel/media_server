use crate::{
    types::{Errors, HttpResponseBytes, Request},
    utils,
};
use http::StatusCode;
use std::collections::HashMap;
use urlencoding::decode;

fn generate_response(
    status_code: u16,
    content_type: &str,
    content: &[u8],
    custom_headers: Option<HashMap<String, String>>,
) -> HttpResponseBytes {
    let status_code =
        StatusCode::from_u16(status_code).expect("Hardcoded status code should be valid");

    let status_line = format!(
        "HTTP/1.1 {} {}\r\n",
        status_code.as_str(),
        status_code
            .canonical_reason()
            .expect("Canonical reason for hardcoded status code should exist")
    );

    let mut headers = String::new();

    if let Some(custom_headers) = custom_headers {
        for (name, value) in custom_headers {
            headers.push_str(&format!("{name}: {value}\r\n"));
        }
    }

    headers.push_str(&format!(
        "Content-Type: {}\r\nContent-Length: {}\r\n\r\n",
        content_type,
        content.len()
    ));

    println!("RESPONSE:");
    println!("{status_line}{}", headers.trim_end());
    println!("----");

    [status_line.as_bytes(), headers.as_bytes(), content].concat()
}

pub fn index(media_dir: &str) -> Result<HttpResponseBytes, Errors> {
    let available_files = utils::list_available_files(media_dir)?.join("\n");

    let response = generate_response(200, "text/plain", available_files.as_bytes(), None);

    Ok(response)
}

pub fn media(request: &Request, route: &str, media_dir: &str) -> Result<HttpResponseBytes, Errors> {
    let desired_filename = decode(route.trim_start_matches("/"))
        .map_err(|_| Errors::ClientError("Unable to decode url".to_owned()))?
        .into_owned();

    let file_content = utils::get_file(media_dir, &desired_filename)?;

    let content_type =
        utils::get_content_type(&desired_filename).map_err(|err| Errors::ClientError(err))?;

    if let Some((bytes_start, bytes_end)) = request.get_range() {
        let file_content_length = file_content.len();
        
        // NOTE: HTTP byte ranges are inclusive, but rust slices are [inclusive, exclusive],
        // so bytes_end represents the rust slice 'exclusive' bound
        let bytes_end = match bytes_end {
            None => file_content_length as u64 - 1,
            Some(end) => {
                if end >= file_content_length as u64 {
                    return Err(Errors::InvalidContentRange);
                }
                end
            }
        };

        let content = &file_content[bytes_start as usize..bytes_end as usize + 1];

        let mut custom_headers = HashMap::new();
        custom_headers.insert("Accept-Ranges".to_owned(), "bytes".to_owned());
        custom_headers.insert(
            "Content-Range".to_owned(),
            format!("bytes {bytes_start}-{bytes_end}/{}", file_content_length),
        );

        Ok(generate_response(
            206,
            &content_type,
            &content,
            Some(custom_headers),
        ))
    } else {
        Ok(generate_response(200, &content_type, &file_content, None))
    }
}

pub fn error(error_code: u16) -> HttpResponseBytes {
    let error_code =
        StatusCode::from_u16(error_code).expect("Hardcoded error code should be valid");

    let error_text = format!(
        "{} {}",
        error_code.as_str(),
        error_code
            .canonical_reason()
            .expect("Hardcoded error code should have canonical reason")
    );

    let response = format!(
        " \
        <!DOCTYPE html> \
        <html lang=\"en\">
        <head> \
            <meta charset=\"utf-8\"> \
            <title>{error_text}</title> \
        </head> \
        <body>
            <h1>{error_text}</h1> \
        </body> \
        </html>"
    );

    generate_response(error_code.as_u16(), "text/html", response.as_bytes(), None)
}

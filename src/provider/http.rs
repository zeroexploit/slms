use std::io::prelude::*;
use std::net::TcpStream;
use std::fs::{metadata, File};
use std::io::SeekFrom;
use chrono::Local;
use chrono::Duration;

use configuration::ServerConfiguration;
use media::Item;

pub enum Status {
    Ok200,
    NotFound404,
    PartialContent206,
    InternalServerError500,
}

impl Status {
    pub fn get(&self) -> String {
        match self {
            &Status::Ok200 => String::from("200 OK"),
            &Status::NotFound404 => String::from("404 Not Found"),
            &Status::PartialContent206 => String::from("206 Partial Content"),
            &Status::InternalServerError500 => String::from("500 Internal Server Error"),
        }
    }
}

pub fn generate_header(
    content_size: usize,
    mime: &str,
    keep_alive: bool,
    server_cfg: &ServerConfiguration,
    status: Status,
) -> String {
    let now = Local::now();

    return format!(
        "HTTP/1.1 {}\r\n\
		 Content-Type: {}\r\n\
		 Content-Length: {}\r\n\
		 Connection: {}\r\n\
		 SID: uuid: {} \r\n\
		 Cache-Control: no-cache\r\n\
		 Date: {}\r\n\
		 Expires: {}\r\n\
		 Server: {}\r\n\r\n",
        status.get(),
        mime,
        content_size,
        if keep_alive { "Keep-Alive" } else { "Close" },
        server_cfg.server_uuid,
        now.format("%a, %d %b %Y %H:%M:%S GMT%z"),
        match now.checked_add_signed(Duration::seconds(180)) {
            Some(result) => result.format("%a, %d %b %Y %H:%M:%S GMT%z"),
            None => now.format("%a, %d %b %Y %H:%M:%S GMT%z"),
        },
        server_cfg.server_tag
    );
}

pub fn stream_file(request: &str, item: &Item, stream: &mut TcpStream, server_tag: &str) {
    // Generate Header
    let mut header: String = String::new();
    let mut bytes_start: u64 = 0;
    let mut bytes_end: u64 = item.file_size;

    match request.to_lowercase().find("range: bytes=") {
        Some(position) => {
            let mut bytes: String = request[(position + 13)..].to_string();
            bytes = bytes[..(bytes.find("\r\n").unwrap())].to_string();

            match bytes.find("-") {
                Some(position) => {
                    if position == 0 {
                        bytes_start = 0;
                        bytes_end = bytes[1..].parse::<u64>().unwrap();
                    } else if position == bytes.len() - 1 {
                        bytes_start = bytes[..position].parse::<u64>().unwrap();
                        bytes_end = item.file_size;
                    } else {
                        bytes_start = bytes[..position].parse::<u64>().unwrap();
                        bytes_end = bytes[position..].parse::<u64>().unwrap();
                    }
                }
                None => {
                    bytes_start = bytes.parse::<u64>().unwrap();
                }
            }

            header.push_str("HTTP/1.1 206 Partial Content\r\n");
            header.push_str("Content-Type: ");
            header.push_str(&item.get_mime_type());
            header.push_str("\r\n");
            header.push_str("Content-Range: bytes ");
            header.push_str(&bytes_start.to_string());
            header.push_str("-");
            header.push_str(&bytes_end.to_string());
            header.push_str("/");
            header.push_str(&item.file_size.to_string());
            header.push_str("\r\n");
            header.push_str("Accept-Ranges: bytes\r\nConnection: close\r\nContentFeatures.DLNA.ORG: DLNA.ORG_OP=11;DLNA.ORG_CI=0\r\nTransferMode.DLNA.ORG: Streaming\r\nServer: ");
            header.push_str(server_tag);
            header.push_str("\r\n");
            header.push_str("Content-Length: ");
            header.push_str(&(bytes_end - bytes_start).to_string());
            header.push_str("\r\n\r\n");

        }
        None => {
            header.push_str("HTTP/1.1 200 OK\r\n");
            header.push_str("Content-Type: ");
            header.push_str(&item.get_mime_type());
            header.push_str("\r\n");
            header.push_str("File-Size: ");
            header.push_str(&item.file_size.to_string());
            header.push_str("\r\n");
            header.push_str("Accept-Ranges: bytes\r\nConnection: close\r\nContentFeatures.DLNA.ORG: DLNA.ORG_OP=11;DLNA.ORG_CI=0\r\nTransferMode.DLNA.ORG: Streaming\r\nServer: ");
            header.push_str(server_tag);
            header.push_str("\r\n");
            header.push_str("Content-Length: ");
            header.push_str(&item.file_size.to_string());
            header.push_str("\r\n\r\n");
        }
    }

    // Send Header
    stream.write(header.as_bytes()).unwrap();

    // Open File
    let mut file = File::open(&item.file_path).unwrap();
    let mut buffer = [0; 65536];
    let mut transferred: u64 = 0;

    file.seek(SeekFrom::Start(bytes_start as u64)).unwrap();

    loop {
        let readed: usize = match file.read(&mut buffer) {
            Ok(read) => read,
            Err(_) => 0,
        };

        if readed == 0 {
            break;
        }
        match stream.write(&buffer[..readed]) {
            Ok(written) => {
                if written != readed {
                    break;
                }
            }
            Err(_) => {
                break;
            }
        }


        transferred += readed as u64;

        if transferred >= bytes_end {
            break;
        }
    }

    stream.flush().unwrap();
}

pub fn send_file(request: &str, path: &str, stream: &mut TcpStream, server_tag: &str, mime: &str) {
    // Generate Header
    let metadata = metadata(path);
    let file_size: u64 = match metadata {
        Ok(some) => some.len(),
        Err(_) => {
            return;
        }
    };
    let mut header = String::new();
    let mut bytes_start: u64 = 0;
    let mut bytes_end: u64 = file_size;

    match request.to_lowercase().find("range: bytes=") {
        Some(position) => {
            let mut bytes: String = request[(position + 13)..].to_string();
            bytes = bytes[..(bytes.find("\r\n").unwrap())].to_string();

            match bytes.find("-") {
                Some(position) => {
                    if position == 0 {
                        bytes_start = 0;
                        bytes_end = bytes[1..].parse::<u64>().unwrap();
                    } else if position == bytes.len() - 1 {
                        bytes_start = bytes[..position].parse::<u64>().unwrap();
                        bytes_end = file_size;
                    } else {
                        bytes_start = bytes[..position].parse::<u64>().unwrap();
                        bytes_end = bytes[position..].parse::<u64>().unwrap();
                    }
                }
                None => {
                    bytes_start = bytes.parse::<u64>().unwrap();
                }
            }

            header.push_str("HTTP/1.1 206 Partial Content\r\n");
            header.push_str("Content-Type: ");
            header.push_str(mime);
            header.push_str("\r\n");
            header.push_str("Content-Range: bytes ");
            header.push_str(&bytes_start.to_string());
            header.push_str("-");
            header.push_str(&bytes_end.to_string());
            header.push_str("/");
            header.push_str(&file_size.to_string());
            header.push_str("\r\n");
            header.push_str("Accept-Ranges: bytes\r\nConnection: close\r\nContentFeatures.DLNA.ORG: DLNA.ORG_OP=11;DLNA.ORG_CI=0\r\nTransferMode.DLNA.ORG: Streaming\r\nServer: ");
            header.push_str(server_tag);
            header.push_str("\r\n");
            header.push_str("Content-Length: ");
            header.push_str(&(bytes_end - bytes_start).to_string());
            header.push_str("\r\n\r\n");

        }
        None => {
            header.push_str("HTTP/1.1 200 OK\r\n");
            header.push_str("Content-Type: ");
            header.push_str(mime);
            header.push_str("\r\n");
            header.push_str("File-Size: ");
            header.push_str(&file_size.to_string());
            header.push_str("\r\n");
            header.push_str("Accept-Ranges: bytes\r\nConnection: close\r\nContentFeatures.DLNA.ORG: DLNA.ORG_OP=11;DLNA.ORG_CI=0\r\nTransferMode.DLNA.ORG: Streaming\r\nServer: ");
            header.push_str(server_tag);
            header.push_str("\r\n");
            header.push_str("Content-Length: ");
            header.push_str(&file_size.to_string());
            header.push_str("\r\n\r\n");
        }
    }

    // Send Header
    stream.write(header.as_bytes()).unwrap();

    // Open File
    let mut file = File::open(path).unwrap();
    let mut buffer = [0; 65536];
    let mut transferred: u64 = 0;

    file.seek(SeekFrom::Start(bytes_start as u64)).unwrap();

    loop {
        let readed: usize = match file.read(&mut buffer) {
            Ok(read) => read,
            Err(_) => 0,
        };

        if readed == 0 {
            break;
        }
        match stream.write(&buffer[..readed]) {
            Ok(written) => {
                if written != readed {
                    break;
                }
            }
            Err(_) => {
                break;
            }
        }


        transferred += readed as u64;

        if transferred >= bytes_end {
            break;
        }
    }

    stream.flush().unwrap();
}

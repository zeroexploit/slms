use std::io::prelude::*;
use std::net::TcpStream;
use std::fs;
use std::fs::File;
use std::io::SeekFrom;

use configuration::ServerConfiguration;
use media::Item;

pub enum Status {
    OK_200,
    NOT_FOUND_404,
    PARTIAL_CONTENT_206,
    INTERNAL_SERVER_ERROR_500,
}

impl Status {
    pub fn get(&self) -> String {
        match self {
            OK_200 => String::from("200 OK"),
            NOT_FOUND_404 => String::from("404 Not Found"),
            PARTIAL_CONTENT_206 => String::from("206 Partial Content"),
            INTERNAL_SERVER_ERROR_500 => String::from("500 Internal Server Error"),
            _ => String::from("500 Internal Server Error"),
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
    let mut header: String = String::from("HTTP/1.1 ");
    header.push_str(&status.get());
    header.push_str("\r\n");
    header.push_str("Content-Type: ");
    header.push_str(mime);
    header.push_str("\r\n");
    header.push_str("Content-Length: ");
    header.push_str(&content_size.to_string());
    header.push_str("\r\n");

    if keep_alive {
        header.push_str("Connection: Keep-Alive\r\n");
    } else {
        header.push_str("Connection: Close\r\n");
    }

    header.push_str("SID: uuid:");
    header.push_str(&server_cfg.server_uuid);
    header.push_str("\r\n");
    header.push_str("Cache-Control: no-cache\r\n");
    header.push_str("Server: ");
    header.push_str(&server_cfg.server_tag);
    header.push_str("\r\n\r\n");

    header
}

pub fn stream_file(request: &str, item: &Item, stream: &mut TcpStream, server_tag: &str) {
    // Generate Header
    let mut header = String::new();
    let mut bytes_start: usize = 0;
    let mut bytes_end: usize = item.file_size as usize;

    match request.to_lowercase().find("range: bytes=") {
        Some(position) => {
            let mut bytes: String = request[(position + 13)..].to_string();
            bytes = bytes[..(bytes.find("\r\n").unwrap())].to_string();

            match bytes.find("-") {
                Some(position) => {
                    if position == 0 {
                        bytes_start = 0;
                        bytes_end = bytes[1..].parse::<usize>().unwrap();
                    } else if position == bytes.len() - 1 {
                        bytes_start = bytes[..position].parse::<usize>().unwrap();
                        bytes_end = item.file_size as usize;
                    } else {
                        bytes_start = bytes[..position].parse::<usize>().unwrap();
                        bytes_end = bytes[position..].parse::<usize>().unwrap();
                    }
                }
                None => {
                    bytes_start = bytes.parse::<usize>().unwrap();
                }
            }

            header = String::from("HTTP/1.1 206 Partial Content\r\n");
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
            header = String::from("HTTP/1.1 200 OK\r\n");
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
    let mut readed: usize = 0;
    let mut transferred: usize = 0;

    file.seek(SeekFrom::Start(bytes_start as u64)).unwrap();

    loop {
        readed = match file.read(&mut buffer) {
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


        transferred += readed;

        if transferred >= bytes_end {
            break;
        }
    }

    stream.flush().unwrap();
}

pub fn send_file(request: &str, path: &str, stream: &mut TcpStream, server_tag: &str, mime: &str) {
    // Generate Header
    let metadata = fs::metadata(path);
    let file_size: usize = match metadata {
        Ok(some) => some.len() as usize,
        Err(_) => {
            return;
        }
    };
    let mut header = String::new();
    let mut bytes_start: usize = 0;
    let mut bytes_end: usize = file_size;

    match request.to_lowercase().find("range: bytes=") {
        Some(position) => {
            let mut bytes: String = request[(position + 13)..].to_string();
            bytes = bytes[..(bytes.find("\r\n").unwrap())].to_string();

            match bytes.find("-") {
                Some(position) => {
                    if position == 0 {
                        bytes_start = 0;
                        bytes_end = bytes[1..].parse::<usize>().unwrap();
                    } else if position == bytes.len() - 1 {
                        bytes_start = bytes[..position].parse::<usize>().unwrap();
                        bytes_end = file_size;
                    } else {
                        bytes_start = bytes[..position].parse::<usize>().unwrap();
                        bytes_end = bytes[position..].parse::<usize>().unwrap();
                    }
                }
                None => {
                    bytes_start = bytes.parse::<usize>().unwrap();
                }
            }

            header = String::from("HTTP/1.1 206 Partial Content\r\n");
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
            header = String::from("HTTP/1.1 200 OK\r\n");
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
    let mut readed: usize = 0;
    let mut transferred: usize = 0;

    file.seek(SeekFrom::Start(bytes_start as u64)).unwrap();

    loop {
        readed = match file.read(&mut buffer) {
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


        transferred += readed;

        if transferred >= bytes_end {
            break;
        }
    }

    stream.flush().unwrap();
}

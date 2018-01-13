use std::io::prelude::*;
use std::net::TcpStream;
use std::io::{SeekFrom, ErrorKind};
use std::fs::{metadata, File};
use chrono::{Local, Duration};

use configuration::ServerConfiguration;

/// # HTTP Status
///
/// Enumeration that Provides the HTTP Status Codes
pub enum Status {
    Ok200,
    BadRequest400,
    Forbidden403,
    NotFound404,
    RangeNotSatisfiable416,
    InternalServerError500,
}

impl Status {
    /// Returns the Statusline as String from the Enumeration Value
    pub fn get(&self) -> String {
        match self {
            &Status::Ok200 => String::from("200 OK"),
            &Status::BadRequest400 => String::from("400 Bad Request"),
            &Status::Forbidden403 => String::from("403 Forbidden"),
            &Status::NotFound404 => String::from("404 Not Found"),
            &Status::RangeNotSatisfiable416 => String::from("416 Range Not Satisfiable"),
            &Status::InternalServerError500 => String::from("500 Internal Server Error"),
        }
    }
}

/// Generates a HTTP Header with the given Values and
/// returns it as String.
///
/// # Arguments
///
/// * `content_size` - Size of the Content in Bytes
/// * `mime` - Mime Type of the Content
/// * `keep_alive` - Kepp the Connection alive or close it
/// * `server_cfg` - Reference to the Server Configuration
/// * `status` - HTTP Status Line as enumeration
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

/// Sends the given Error Code to the Stream.
/// No Content, Header only.
///
/// # Arguments
///
/// * `status` - HTTP Status Code as enumeration
/// * `server_cfg` - Reference to the Server Configuration
/// * `stream` - TcpStream so send the Header to
pub fn send_error(status: Status, server_cfg: &ServerConfiguration, stream: &mut TcpStream) {
    match stream.write(
        generate_header(0, "text/html", false, server_cfg, status).as_bytes(),
    ) {
        Ok(_) => stream.flush().unwrap_or(()),
        Err(_) => {}
    }
}

/// Sends the given File to the remote Host supporting Byte Ranges.
/// If something goes wrong it will return immeditly.
///
/// # Arguments
///
/// * `request` - The original incoming Request (Http Header)
/// * `path` - Path to the File to serve
/// * `stream` - TcpStream to write to
/// * `server_cfg` - Reference to the Server Configuration to use
/// * `mime` - Mime Type to use
pub fn send_file(
    request: &str,
    path: &str,
    stream: &mut TcpStream,
    server_cfg: &ServerConfiguration,
    mime: &str,
) {
    // Generate Header
    let metadata = metadata(path);
    let file_size: u64 =
        match metadata {
            Ok(some) => some.len(),
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => send_error(Status::NotFound404, server_cfg, stream),
                    ErrorKind::PermissionDenied => {
                        send_error(Status::Forbidden403, server_cfg, stream)
                    } 
                    _ => send_error(Status::InternalServerError500, server_cfg, stream),
                }

                return;
            }
        };
    let mut header: String = String::new();
    let mut bytes_start: u64 = 0;
    let mut bytes_end: u64 = file_size;

    // Calculate the Byte offsets if requested
    match request.to_lowercase().find("range: bytes=") {
        Some(position) => {
            // Make sure the Requests Line is complete -> Send Error if not
            if position + 13 >= request.len() {

                send_error(Status::BadRequest400, server_cfg, stream);
                return;
            }

            // Extract the bytes Request -> Send Error if something fails here
            let mut bytes: String = request[(position + 13)..].to_string();
            bytes = bytes[..(match bytes.find("\r\n") {
                                 Some(position) => position,
                                 None => {
                                     send_error(Status::BadRequest400, server_cfg, stream);
                                     return;
                                 }
                             })].to_string();

            // Handle possibilities ("start-end" "-end" "start-" "start")
            match bytes.find("-") {
                Some(position) => {
                    if position == 0 {
                        bytes_start = 0;
                        bytes_end = match bytes[1..].parse::<u64>() {
                            Ok(value) => value,
                            Err(_) => {
                                send_error(Status::BadRequest400, server_cfg, stream);
                                return;
                            }
                        }
                    } else if position == bytes.len() - 1 {
                        bytes_start = match bytes[..position].parse::<u64>() {
                            Ok(value) => value,
                            Err(_) => {
                                send_error(Status::BadRequest400, server_cfg, stream);
                                return;
                            }
                        };
                        bytes_end = file_size;
                    } else {
                        bytes_start = match bytes[..position].parse::<u64>() {
                            Ok(value) => value,
                            Err(_) => {
                                send_error(Status::BadRequest400, server_cfg, stream);
                                return;
                            }
                        };

                        bytes_end = match bytes[position..].parse::<u64>() {
                            Ok(value) => value,
                            Err(_) => {
                                send_error(Status::BadRequest400, server_cfg, stream);
                                return;
                            }
                        }
                    }
                }
                None => {
                    bytes_start = match bytes.parse::<u64>() {
                        Ok(value) => value,
                        Err(_) => {
                            send_error(Status::BadRequest400, server_cfg, stream);
                            return;
                        }
                    };
                }
            }
            // Check Boundarys are in File
            if bytes_start > bytes_end || bytes_end > file_size {
                send_error(Status::RangeNotSatisfiable416, server_cfg, stream);
                return;
            }
            // Create Partial Content Header
            header.push_str(&format!(
                "HTTP/1.1 206 Partial Content\r\n\
            	 Content-Type: {}\r\n\
            	 Content-Range: bytes {}-{}/{}\r\n\
            	 Accept-Ranges: bytes\r\n\
            	 Connection: Close\r\n\
            	 ContentFeatures.DLNA.ORG: DLNA.ORG_OP=11;DLNA.ORG_CI=0\r\n\
            	 TransferMode.DLNA.ORG: Streaming\r\n\
            	 Server: {}\r\n\
            	 Content-Length: {}\r\n\r\n",
                mime,
                bytes_start,
                bytes_end,
                file_size,
                server_cfg.server_tag,
                bytes_end - bytes_start
            ));
        }
        None => {
            // Create default Header
            header.push_str(&format!(
                "HTTP/1.1 200 OK\r\n\
	             Content-Type: {}\r\n\
	             File-Size: {}\r\n\
	             Accept-Ranges: bytes\r\n\
            	 Connection: Close\r\n\
            	 ContentFeatures.DLNA.ORG: DLNA.ORG_OP=11;DLNA.ORG_CI=0\r\n\
            	 TransferMode.DLNA.ORG: Streaming\r\n\
            	 Server: {}\r\n\
            	 Content-Length: {}\r\n\r\n",
                mime,
                file_size,
                server_cfg.server_tag,
                file_size
            ));

            info!("Streaming {} to {}", path, stream.peer_addr().unwrap());
        }
    }
    // Send Header
    match stream.write_all(header.as_bytes()) {
        Ok(_) => {}
        Err(_) => {
            return;
        }
    }
    // Open File and create Buffer
    let mut file =
        match File::open(path) {
            Ok(value) => value,
            Err(err) => {
                match err.kind() {
                    ErrorKind::NotFound => send_error(Status::NotFound404, server_cfg, stream),
                    ErrorKind::PermissionDenied => {
                        send_error(Status::Forbidden403, server_cfg, stream)
                    } 
                    _ => send_error(Status::InternalServerError500, server_cfg, stream),
                }

                return;
            }
        };
    let mut buffer = [0; 65515];
    let mut transferred: u64 = 0;

    // Seek to requested Position
    match file.seek(SeekFrom::Start(bytes_start)) {
        Ok(_) => {}
        Err(_) => {
            send_error(Status::InternalServerError500, server_cfg, stream);
            return;
        }
    }
    // Send the File Contents
    loop {
        // Read from File
        let readed: usize = match file.read(&mut buffer) {
            Ok(read) => read,
            Err(_) => {
                break;
            }
        };

        if readed == 0 {
            break;
        }

        // Write to Stream
        match stream.write_all(&buffer[..readed]) {
            Ok(_) => {}
            Err(_) => {
                break;
            }
        }

        // Abort if everything was sent
        transferred += readed as u64;

        if transferred >= bytes_end {
            break;
        }

    }

    // Make sure everything is transferred
    match stream.flush() {
        Ok(_) => {}
        Err(_) => {}
    }
}

use configuration::ServerConfiguration;

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

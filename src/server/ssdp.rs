use std::net::UdpSocket;
use std::str;
use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::thread;
use std::time;

use configuration::ServerConfiguration;

/// # SSDP Server
///
/// Basic Implementation of the SSDP
/// for announcing the Media Server inside the
/// Network.
/// Spawns an individual Thread for Operation in Order
/// to not block Main Execution.
///
/// # TO-DO
/// - Implement Error Handling
pub struct SSDPServer<'a> {
    server_cfg: &'a ServerConfiguration,
    socket: UdpSocket,
}

impl<'a> SSDPServer<'a> {
    /// Creates new Instance of the SSDP Server and directly
    /// binds the required UDP Sockets.
    pub fn new(server_cfg: &ServerConfiguration) -> SSDPServer {
        SSDPServer {
            server_cfg,
            socket: UdpSocket::bind(("0.0.0.0", 1900)).unwrap(),
        }
    }

    /// Brings up the SSDP Server and announces the UPnP Services
    /// to the Network.
    /// Runs in an new Thread and spawns another to periodically
    /// announce the Services to the Network.
    pub fn discover(&self) {
        self.socket.set_multicast_loop_v4(false).expect("");
        self.socket
            .join_multicast_v4(
                &Ipv4Addr::new(239, 255, 255, 250),
                &self.server_cfg.server_ip.parse::<Ipv4Addr>().unwrap(),
            )
            .expect("");

        let socket_c = self.socket.try_clone().unwrap();
        let cfg_ip = self.server_cfg.server_ip.clone();
        let cfg_port = self.server_cfg.server_port;
        let cfg_uuid = self.server_cfg.server_uuid.clone();
        let cfg_tag = self.server_cfg.server_tag.clone();

        thread::spawn(move || {
            SSDPServer::send_notify_packages(
                socket_c,
                &cfg_uuid,
                &cfg_ip,
                &cfg_port.to_string(),
                &cfg_tag,
            );
        });

        let socket_c = self.socket.try_clone().unwrap();
        let cfg_ip = self.server_cfg.server_ip.clone();
        let cfg_port = self.server_cfg.server_port;
        let cfg_uuid = self.server_cfg.server_uuid.clone();
        let cfg_tag = self.server_cfg.server_tag.clone();

        thread::spawn(move || {
            let mut buffer = [0; 2048];

            loop {
                let (amt, src) = socket_c.recv_from(&mut buffer).unwrap();

                if amt > 0 {
                    let request = str::from_utf8(&buffer[..amt]).unwrap();

                    if &request[..10] == "M-SEARCH *" {
                        SSDPServer::send_search_response(
                            socket_c.try_clone().unwrap(),
                            src,
                            request,
                            &cfg_uuid,
                            &cfg_ip,
                            &cfg_port.to_string(),
                            &cfg_tag,
                        );
                    }
                }
            }
        });
    }

    /// Removes the Services from the Network using ByeBye Signals.
    /// This will stop the SSDP Server.
    pub fn byebye(&self) {
        let maddr = SocketAddr::new("239.255.255.250".parse::<IpAddr>().unwrap(), 1900);

        self.socket
            .send_to(
                SSDPServer::get_notify_package(
                    "byebye",
                    "upnp:rootdevice",
                    &("uuid:".to_string() + &self.server_cfg.server_uuid +
                          &"::upnp:rootdevice".to_string()),
                    "description.xml",
                    &self.server_cfg.server_ip,
                    &self.server_cfg.server_port.to_string(),
                    &self.server_cfg.server_tag,
                ).as_bytes(),
                maddr,
            )
            .expect("");

        self.socket
            .send_to(
                SSDPServer::get_notify_package(
                    "byebye",
                    &("uuid:".to_string() + &self.server_cfg.server_uuid),
                    &("uuid:".to_string() + &self.server_cfg.server_uuid),
                    "description.xml",
                    &self.server_cfg.server_ip,
                    &self.server_cfg.server_port.to_string(),
                    &self.server_cfg.server_tag,
                ).as_bytes(),
                maddr,
            )
            .expect("");

        self.socket
            .send_to(
                SSDPServer::get_notify_package(
                    "byebye",
                    "urn:schemas-upnp-org:device:MediaServer:1",
                    &("uuid:".to_string() + &self.server_cfg.server_uuid +
                          &"::urn:schemas-upnp-org:device:MediaServer:1".to_string()),
                    "description.xml",
                    &self.server_cfg.server_ip,
                    &self.server_cfg.server_port.to_string(),
                    &self.server_cfg.server_tag,
                ).as_bytes(),
                maddr,
            )
            .expect("");

        self.socket
            .send_to(
                SSDPServer::get_notify_package(
                    "byebye",
                    "urn:schemas-upnp-org:service:ContentDirectory:1",
                    &("USN: uuid:".to_string() + &self.server_cfg.server_uuid +
                          &"::urn:schemas-upnp-org:service:ContentDirectory:1".to_string()),
                    "description.xml",
                    &self.server_cfg.server_ip,
                    &self.server_cfg.server_port.to_string(),
                    &self.server_cfg.server_tag,
                ).as_bytes(),
                maddr,
            )
            .expect("");

        self.socket
            .send_to(
                SSDPServer::get_notify_package(
                    "byebye",
                    "urn:schemas-upnp-org:service:ConnectionManager:1",
                    &("USN: uuid:".to_string() + &self.server_cfg.server_uuid +
                          &"::urn:schemas-upnp-org:service:ConnectionManager:1".to_string()),
                    "description.xml",
                    &self.server_cfg.server_ip,
                    &self.server_cfg.server_port.to_string(),
                    &self.server_cfg.server_tag,
                ).as_bytes(),
                maddr,
            )
            .expect("");
    }

    /// Periodically sends Notify Packages to the Network
    /// to announce the Services to the Network.
    /// New alive Messages will be send every 180s
    fn send_notify_packages(socket: UdpSocket, uuid: &str, ip: &str, port: &str, tag: &str) {
        loop {
            let maddr = SocketAddr::new("239.255.255.250".parse::<IpAddr>().unwrap(), 1900);

            socket
                .send_to(
                    SSDPServer::get_notify_package(
                        "alive",
                        "upnp:rootdevice",
                        &("uuid:".to_string() + uuid + &"::upnp:rootdevice".to_string()),
                        "description.xml",
                        ip,
                        port,
                        tag,
                    ).as_bytes(),
                    maddr,
                )
                .expect("");

            socket
                .send_to(
                    SSDPServer::get_notify_package(
                        "alive",
                        &("uuid:".to_string() + uuid),
                        &("uuid:".to_string() + uuid),
                        "description.xml",
                        ip,
                        port,
                        tag,
                    ).as_bytes(),
                    maddr,
                )
                .expect("");

            socket
                .send_to(
                    SSDPServer::get_notify_package(
                        "alive",
                        "urn:schemas-upnp-org:device:MediaServer:1",
                        &("uuid:".to_string() + &uuid +
                              &"::urn:schemas-upnp-org:device:MediaServer:1".to_string()),
                        "description.xml",
                        ip,
                        port,
                        tag,
                    ).as_bytes(),
                    maddr,
                )
                .expect("");

            socket
                .send_to(
                    SSDPServer::get_notify_package(
                        "alive",
                        "urn:schemas-upnp-org:service:ContentDirectory:1",
                        &("USN: uuid:".to_string() + uuid +
                              &"::urn:schemas-upnp-org:service:ContentDirectory:1".to_string()),
                        "description.xml",
                        ip,
                        port,
                        tag,
                    ).as_bytes(),
                    maddr,
                )
                .expect("");

            socket
                .send_to(
                    SSDPServer::get_notify_package(
                        "alive",
                        "urn:schemas-upnp-org:service:ConnectionManager:1",
                        &("USN: uuid:".to_string() + uuid +
                              &"::urn:schemas-upnp-org:service:ConnectionManager:1".to_string()),
                        "description.xml",
                        ip,
                        port,
                        tag,
                    ).as_bytes(),
                    maddr,
                )
                .expect("");

            thread::sleep(time::Duration::from_secs(180));
        }
    }

    /// Send answers to Search Requests
    fn send_search_response(
        socket: UdpSocket,
        receiver: SocketAddr,
        request: &str,
        uuid: &str,
        ip: &str,
        port: &str,
        tag: &str,
    ) {
        let mut message = String::new();

        if request.find("ssdp:all").is_some() {
            message = SSDPServer::get_search_response_package(
                "urn:schemas-upnp-org:device:MediaServer:1",
                &("uuid:".to_string() + uuid +
                      &"::urn:schemas-upnp-org:device:MediaServer:1".to_string()),
                "description.xml",
                ip,
                port,
                tag,
            )
        } else if request.find("upnp:rootdevice").is_some() {
            message = SSDPServer::get_search_response_package(
                "upnp:rootdevice",
                &("uuid:".to_string() + uuid + &"::upnp:rootdevice".to_string()),
                "description.xml",
                ip,
                port,
                tag,
            );
        } else if request
                   .find("urn:schemas-upnp-org:device:MediaServer")
                   .is_some()
        {
            message = SSDPServer::get_search_response_package(
                "urn:schemas-upnp-org:device:MediaServer:1",
                &("uuid:".to_string() + uuid +
                      &"::urn:schemas-upnp-org:device:MediaServer:1".to_string()),
                "description.xml",
                ip,
                port,
                tag,
            )
        } else if request
                   .find("urn:schemas-upnp-org:service:ContentDirectory")
                   .is_some()
        {
            message = SSDPServer::get_search_response_package(
                "urn:schemas-upnp-org:service:ContentDirectory:1",
                &("uuid:".to_string() + uuid +
                      &"::urn:schemas-upnp-org:service:ContentDirectory:1".to_string()),
                "description.xml",
                ip,
                port,
                tag,
            )
        } else if request
                   .find("urn:schemas-upnp-org:service:ConnectionManager")
                   .is_some()
        {
            message = SSDPServer::get_search_response_package(
                "urn:schemas-upnp-org:service:ConnectionManager:1",
                &("uuid:".to_string() + uuid +
                      &"::urn:schemas-upnp-org:service:ConnectionManager:1".to_string()),
                "description.xml",
                ip,
                port,
                tag,
            )
        } else if request
                   .find("urn:microsoft.com:service:X_MS_MediaReceiverRegistrar")
                   .is_some()
        {
            message = SSDPServer::get_search_response_package(
                "urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1",
                &("uuid:".to_string() + uuid +
                      &"::urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1".to_string()),
                "description.xml",
                ip,
                port,
                tag,
            )
        }

        if message.len() > 0 {
            socket.send_to(message.as_bytes(), receiver).expect("");
        }
    }

    /// Generate a Notify Package for sending over UDP
    fn get_notify_package(
        nts: &str,
        nt: &str,
        usn: &str,
        location: &str,
        server_ip: &str,
        server_port: &str,
        server_tag: &str,
    ) -> String {
        let mut header: String = String::new();

        header.push_str("NOTIFY * HTTP/1.1\r\n");
        header.push_str("SERVER: ");
        header.push_str(server_tag);
        header.push_str("\r\n");
        header.push_str("CACHE-CONTROL: max-age=1800\r\n");
        header.push_str("LOCATION: http://");
        header.push_str(server_ip);
        header.push_str(":");
        header.push_str(server_port);
        header.push_str("/connection/");
        header.push_str(location);
        header.push_str("\r\n");
        header.push_str("NTS: ssdp:");
        header.push_str(nts);
        header.push_str("\r\n");
        header.push_str("NT: ");
        header.push_str(nt);
        header.push_str("\r\n");
        header.push_str("USN: ");
        header.push_str(usn);
        header.push_str("\r\n");
        header.push_str("HOST: 239.255.255.250:1900");
        header.push_str("\r\n\r\n");

        header
    }

    /// Generate a Search Response Package as Answer to
    /// a M-SEARCH Request
    fn get_search_response_package(
        st: &str,
        usn: &str,
        location: &str,
        server_ip: &str,
        server_port: &str,
        server_tag: &str,
    ) -> String {
        let mut response: String = String::new();

        response.push_str("HTTP/1.1 200 OK\r\n");
        response.push_str("SERVER: ");
        response.push_str(server_tag);
        response.push_str("\r\n");
        response.push_str("CACHE-CONTROL: max-age=1800\r\n");
        response.push_str("LOCATION: http://");
        response.push_str(server_ip);
        response.push_str(":");
        response.push_str(server_port);
        response.push_str("/connection/");
        response.push_str(location);
        response.push_str("\r\n");
        response.push_str("ST: ");
        response.push_str(st);
        response.push_str("\r\n");
        response.push_str("USN: ");
        response.push_str(usn);
        response.push_str("\r\n");
        response.push_str("Content-Length: 0\r\n");
        response.push_str("EXT:\r\n\r\n");

        response
    }
}

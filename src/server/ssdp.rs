use std::net::{UdpSocket, SocketAddr, Ipv4Addr, IpAddr};
use std::{str, time, thread};

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
/// - Bind to configurated Interface
pub struct SSDPServer<'a> {
    server_cfg: &'a ServerConfiguration,
    socket: UdpSocket,
}

impl<'a> SSDPServer<'a> {
    /// Creates new Instance of the SSDP Server and directly
    /// binds the required UDP Sockets. Returns Err if unable to bind
    ///
    /// # Arguments
    ///
    /// * `server_cfg` - The Servers Configuration
    pub fn new(server_cfg: &ServerConfiguration) -> Result<SSDPServer, ()> {
        let ssdp = SSDPServer {
            server_cfg,
            socket: match UdpSocket::bind(("0.0.0.0", 1900)) {
                Ok(value) => value,
                Err(_) => return Err(()),
            },
        };

        Ok(ssdp)
    }

    /// Brings up the SSDP Server and announces the UPnP Services
    /// to the Network.
    /// Runs in an new Thread and spawns another to periodically
    /// announce the Services to the Network.
    pub fn discover(&self) -> Result<(), ()> {
        match self.socket.set_multicast_loop_v4(false) {
            Ok(_) => {}
            Err(_) => return Err(()),
        };

        match self.socket.join_multicast_v4(
            &Ipv4Addr::new(239, 255, 255, 250),
            &match self.server_cfg.server_ip.parse::<Ipv4Addr>() {
                Ok(value) => value,
                Err(_) => return Err(()),
            },
        ) {
            Ok(_) => {}
            Err(_) => return Err(()),
        };

        let socket_c = match self.socket.try_clone() {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
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

        let socket_c = match self.socket.try_clone() {
            Ok(value) => value,
            Err(_) => return Err(()),
        };
        let cfg_ip = self.server_cfg.server_ip.clone();
        let cfg_port = self.server_cfg.server_port;
        let cfg_uuid = self.server_cfg.server_uuid.clone();
        let cfg_tag = self.server_cfg.server_tag.clone();

        thread::spawn(move || {
            let mut buffer = [0; 4096];

            loop {
                let (amt, src) = match socket_c.recv_from(&mut buffer) {
                    Ok(value) => value,
                    Err(_) => break,
                };

                if amt > 0 {
                    let request = match str::from_utf8(&buffer[..amt]) {
                        Ok(value) => value,
                        Err(_) => break,
                    };

                    if request.len() > 10 && &request[..10] == "M-SEARCH *" {
                        SSDPServer::send_search_response(
                            match socket_c.try_clone() {
                                Ok(value) => value,
                                Err(_) => break,
                            },
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

        Ok(())
    }

    /// Removes the Services from the Network using ByeBye Signals.
    /// This will stop the SSDP Server.
    pub fn byebye(&self) {
        let maddr = SocketAddr::new(
            match "239.255.255.250".parse::<IpAddr>() {
                Ok(value) => value,
                Err(_) => return,
            },
            1900,
        );

        match self.socket.send_to(
            SSDPServer::get_notify_package(
                "byebye",
                "upnp:rootdevice",
                &format!("uuid:{}::upnp:rootdevice", self.server_cfg.server_uuid),
                "description.xml",
                &self.server_cfg.server_ip,
                &self.server_cfg.server_port.to_string(),
                &self.server_cfg.server_tag,
            ).as_bytes(),
            maddr,
        ) {
            Ok(_) => {}
            Err(_) => return,
        };

        match self.socket.send_to(
            SSDPServer::get_notify_package(
                "byebye",
                &format!("uuid:{}", self.server_cfg.server_uuid),
                &format!("uuid:{}", self.server_cfg.server_uuid),
                "description.xml",
                &self.server_cfg.server_ip,
                &self.server_cfg.server_port.to_string(),
                &self.server_cfg.server_tag,
            ).as_bytes(),
            maddr,
        ) {
            Ok(_) => {}
            Err(_) => return,
        };

        match self.socket.send_to(
            SSDPServer::get_notify_package(
                "byebye",
                "urn:schemas-upnp-org:device:MediaServer:1",
                &format!(
                    "uuid:{}::urn:schemas-upnp-org:device:MediaServer:1",
                    self.server_cfg.server_uuid
                ),
                "description.xml",
                &self.server_cfg.server_ip,
                &self.server_cfg.server_port.to_string(),
                &self.server_cfg.server_tag,
            ).as_bytes(),
            maddr,
        ) {
            Ok(_) => {}
            Err(_) => return,
        };

        match self.socket.send_to(
            SSDPServer::get_notify_package(
                "byebye",
                "urn:schemas-upnp-org:service:ContentDirectory:1",
                &format!(
                    "USN: uuid:{}::urn:schemas-upnp-org:service:ContentDirectory:1",
                    self.server_cfg.server_uuid
                ),
                "description.xml",
                &self.server_cfg.server_ip,
                &self.server_cfg.server_port.to_string(),
                &self.server_cfg.server_tag,
            ).as_bytes(),
            maddr,
        ) {
            Ok(_) => {}
            Err(_) => return,
        };

        match self.socket.send_to(
            SSDPServer::get_notify_package(
                "byebye",
                "urn:schemas-upnp-org:service:ConnectionManager:1",
                &format!(
                    "USN: uuid:{}::urn:schemas-upnp-org:service:ConnectionManager:1",
                    self.server_cfg.server_uuid
                ),
                "description.xml",
                &self.server_cfg.server_ip,
                &self.server_cfg.server_port.to_string(),
                &self.server_cfg.server_tag,
            ).as_bytes(),
            maddr,
        ) {
            Ok(_) => {}
            Err(_) => return,
        };
    }

    /// Periodically sends Notify Packages to the Network
    /// to announce the Services to the Network.
    /// New alive Messages will be send every 180s
    ///
    /// # Arguments
    ///
    /// * `socket` - UDP Socket to use
    /// * `uuid` - UUID of the Media Server
    /// * `ip` - IP Address of the Server
    /// * `port` - Port the Server listens on
    /// * `tag` - The Servers HTTP Server Header Tag
    fn send_notify_packages(socket: UdpSocket, uuid: &str, ip: &str, port: &str, tag: &str) {
        loop {
            let maddr = SocketAddr::new(
                match "239.255.255.250".parse::<IpAddr>() {
                    Ok(value) => value,
                    Err(_) => return,
                },
                1900,
            );

            match socket.send_to(
                SSDPServer::get_notify_package(
                    "alive",
                    "upnp:rootdevice",
                    &format!("uuid:{}::upnp:rootdevice", uuid),
                    "description.xml",
                    ip,
                    port,
                    tag,
                ).as_bytes(),
                maddr,
            ) {
                Ok(_) => {}
                Err(_) => return,
            };

            match socket.send_to(
                SSDPServer::get_notify_package(
                    "alive",
                    &format!("uuid:{}", uuid),
                    &format!("uuid:{}", uuid),
                    "description.xml",
                    ip,
                    port,
                    tag,
                ).as_bytes(),
                maddr,
            ) {
                Ok(_) => {}
                Err(_) => return,
            };

            match socket.send_to(
                SSDPServer::get_notify_package(
                    "alive",
                    "urn:schemas-upnp-org:device:MediaServer:1",
                    &format!("uuid:{}::urn:schemas-upnp-org:device:MediaServer:1", uuid),
                    "description.xml",
                    ip,
                    port,
                    tag,
                ).as_bytes(),
                maddr,
            ) {
                Ok(_) => {}
                Err(_) => return,
            };

            match socket.send_to(
                SSDPServer::get_notify_package(
                    "alive",
                    "urn:schemas-upnp-org:service:ContentDirectory:1",
                    &format!(
                        "USN: uuid:{}::urn:schemas-upnp-org:service:ContentDirectory:1",
                        uuid
                    ),
                    "description.xml",
                    ip,
                    port,
                    tag,
                ).as_bytes(),
                maddr,
            ) {
                Ok(_) => {}
                Err(_) => return,
            };

            match socket.send_to(
                SSDPServer::get_notify_package(
                    "alive",
                    "urn:schemas-upnp-org:service:ConnectionManager:1",
                    &format!(
                        "USN: uuid:{}::urn:schemas-upnp-org:service:ConnectionManager:1",
                        uuid
                    ),
                    "description.xml",
                    ip,
                    port,
                    tag,
                ).as_bytes(),
                maddr,
            ) {
                Ok(_) => {}
                Err(_) => return,
            };

            thread::sleep(time::Duration::from_secs(180));
        }
    }

    /// Send answers to Search Requests
    ///
    /// # Arguments
    ///
    /// * `socket` - The Socket to send on
    /// * `receiver` - Address of the Receiver to send to
    /// * `request` - The received Search Request
    /// * `uuid` - UUID of the Media Server
    /// * `ip` - IP Address of the Server
    /// * `port` - Port the Server listens on
    /// * `tag` - The Servers HTTP Server Header Tag
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
                &format!("uuid:{}::urn:schemas-upnp-org:device:MediaServer:1", uuid),
                "description.xml",
                ip,
                port,
                tag,
            )
        } else if request.find("upnp:rootdevice").is_some() {
            message = SSDPServer::get_search_response_package(
                "upnp:rootdevice",
                &format!("uuid:{}::upnp:rootdevice", uuid),
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
                &format!("uuid:{}::urn:schemas-upnp-org:device:MediaServer:1", uuid),
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
                &format!(
                    "uuid:{}::urn:schemas-upnp-org:service:ContentDirectory:1",
                    uuid
                ),
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
                &format!(
                    "uuid:{}::urn:schemas-upnp-org:service:ConnectionManager:1",
                    uuid
                ),
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
                &format!(
                    "uuid:{}::urn:microsoft.com:service:X_MS_MediaReceiverRegistrar:1",
                    uuid
                ),
                "description.xml",
                ip,
                port,
                tag,
            )
        }

        if message.len() > 0 {
            match socket.send_to(message.as_bytes(), receiver) {
                Ok(_) => {}
                Err(_) => return,
            };
        }
    }

    /// Generate a Notify Package for sending over UDP
    ///
    /// # Arguments
    ///
    /// * `nts` - NTS Value to use
    /// * `nt` - NT Value to use
    /// * `usn` - USN Value to use
    /// * `location` - Path to send
    /// * `server_ip` - IP of the Server
    /// * `server_port` - Port the Server listens on
    /// * `server_tag` - The Servers HTTP Identification Tag
    fn get_notify_package(
        nts: &str,
        nt: &str,
        usn: &str,
        location: &str,
        server_ip: &str,
        server_port: &str,
        server_tag: &str,
    ) -> String {
        format!(
        	"NOTIFY * HTTP/1.1\r\n\
        	SERVER: {}\r\n\
        	CACHE-CONTROL: max-age=1800\r\n\
        	LOCATION: http://{}:{}/connection/{}\r\n\
        	NTS: ssdp: {}\r\n\
        	NT: {}\r\n\
        	USN: {}\r\n\
        	HOST: 239.255.255.250:1900\r\n\r\n",
        server_tag,
        server_ip,
        server_port,
        location,
        nts,
        nt,
        usn,
        )
    }

    /// Generate a Search Response Package as Answer to
    /// a M-SEARCH Request ///
    /// # Arguments
    ///
    /// * `st` - ST Value to use
    /// * `usn` - USN Value to use
    /// * `location` - Path to send
    /// * `server_ip` - IP of the Server
    /// * `server_port` - Port the Server listens on
    /// * `server_tag` - The Servers HTTP Identification Tag
    fn get_search_response_package(
        st: &str,
        usn: &str,
        location: &str,
        server_ip: &str,
        server_port: &str,
        server_tag: &str,
    ) -> String {
        format!(
        "HTTP/1.1 200 OK\r\n\
        SERVER: {}\r\n\
        CACHE-CONTROL: max-age=1800\r\n\
        LOCATION: http://{}:{}/connection/{}\r\n\
        ST: {}\r\n\
        USN: {}\r\n\
        Content-Length: 0\r\n\
        EXT:\r\n\r\n",
        server_tag,
        server_ip,
        server_port,
        location,
        st,
        usn,)
    }
}

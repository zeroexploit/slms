use std::net::TcpListener;
use std::thread;
use std::sync::Mutex;
use daemonize::Daemonize;
use std::net::TcpStream;

use configuration::{ConfigurationHandler, ServerConfiguration};
use tools::{Logger, LogLevel};
use database::DatabaseManager;
use server::SSDPServer;
use upnp::{ConnectionManager, ContentDirectory};
use provider::http;


lazy_static! { static ref DB_MANAGER: Mutex<DatabaseManager> = Mutex::new(DatabaseManager::new()); }

pub struct MediaServer {}

impl MediaServer {
    pub fn run(cfg_path: &str, daemonize: bool, helpscreen: bool) {
        // Show Welcome Text
        MediaServer::print_welcome();

        // Show help and abort if requested
        if helpscreen {
            MediaServer::print_helpscreen();
            return;
        }

        // Try to parse the Configurations
        let mut cfg_handler: ConfigurationHandler = ConfigurationHandler::new();

        if !cfg_handler.parse(cfg_path) {
            println!(
                "Unable to load the Configuration File: {} !! Make sure to make it readable !! - Shutdown",
                cfg_path
            );

            return;
        }

        // Prepare Logging
        let logger: Logger = Logger::new(
            &cfg_handler.server_configuration.log_path,
            cfg_handler.server_configuration.log_level,
        );

        // Bring to Background if requested
        if daemonize {
            println!("Moving Application to the Background...");
            let daemonize = Daemonize::new();

            match daemonize.start() {
                Err(_) => return,
                Ok(_) => {
                    println!("Unable to daemonize! Aborting..");
                }
            }
        }

        // Ouput to Log File
        logger.write_log(
            &format!(
                "Simple Linux Media Server Version: {}",
                option_env!("CARGO_PKG_VERSION").unwrap_or("")
            ),
            LogLevel::INFORMATION,
        );

        logger.write_log("Loading Database...", LogLevel::INFORMATION);

        // Bring up the Media Database
        match DB_MANAGER.lock() {
            Ok(mut value) => {
                value.load(
                    &cfg_handler.server_configuration.media_db_path,
                    cfg_handler.server_configuration.share_dirs.clone(),
                )
            }
            Err(_) => {
                logger.write_log("Unable to get Database Mutex - db.load()!", LogLevel::ERROR);
                return;
            }
        }

        match DB_MANAGER.lock() {
            Ok(mut value) => value.boot_up(),
            Err(_) => {
                logger.write_log(
                    "Unable to get Database Mutex - db.boot_up()!",
                    LogLevel::ERROR,
                );
                return;
            }
        }

        logger.write_log("Database ready.", LogLevel::INFORMATION);

        // Prepare the Sockets
        let listener = match TcpListener::bind((
            cfg_handler.server_configuration.server_ip.as_str(),
            cfg_handler.server_configuration.server_port,
        )) {
            Ok(value) => value,
            Err(_) => {
                logger.write_log(
                    &format!(
                        "Unable to bind TCP Socket to {}:{}!",
                        cfg_handler.server_configuration.server_ip,
                        cfg_handler.server_configuration.server_port
                    ),
                    LogLevel::ERROR,
                );
                return;
            }
        };

        logger.write_log("Running SSDP Server...", LogLevel::INFORMATION);

        // Bring up the SSDP Server
        let ssdp_server: SSDPServer = match SSDPServer::new(&cfg_handler.server_configuration) {
            Ok(value) => value,
            Err(_) => {
                logger.write_log("Unable to create SSDP Server!", LogLevel::ERROR);
                return;
            }
        };

        match ssdp_server.discover() {
            Ok(_) => {}
            Err(_) => {
                logger.write_log("Unable to announce Server!", LogLevel::ERROR);
                return;
            }
        }

        logger.write_log("Waiting for incoming Connections...", LogLevel::INFORMATION);

        // Process Incoming Connections in new Threads
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(value) => value,
                Err(_) => {
                    logger.write_log("Unable to establish Connection!", LogLevel::ERROR);
                    break;
                }
            };
            let tcfg_handler = cfg_handler.clone();
            let svr_cfg = cfg_handler.server_configuration.clone();

            logger.write_log(
                &format!(
                    "New Connection from: {}",
                    match stream.peer_addr() {
                        Ok(value) => value,
                        Err(_) => continue,
                    }
                ),
                LogLevel::VERBOSE,
            );

            thread::spawn(move || {
                MediaServer::process_incoming(&mut stream, &svr_cfg, &tcfg_handler);
            });
        }

        logger.write_log("Something went wrong. Shutting down!", LogLevel::ERROR);

        // Clean Up
        ssdp_server.byebye();
    }

    fn process_incoming(
        stream: &mut TcpStream,
        svr_cfg: &ServerConfiguration,
        tcfg_handler: &ConfigurationHandler,
    ) {
        let con_manager: ConnectionManager = ConnectionManager::new(&svr_cfg);
        let content: String = con_manager.handle_connection(stream);
        let mut xml: String = String::new();

        if content.find("/connection/").is_some() {
            xml = con_manager.handle_request(&content);
        } else if content.find("/content/").is_some() {
            let db = DB_MANAGER.lock().unwrap();
            let mut con_dir: ContentDirectory = ContentDirectory::new(&tcfg_handler, &db);
            xml = con_dir.handle_request(&content);
        } else if content.find("/stream/").is_some() {
            // Streaming
            let id_field: &str = &content[(content.find("/stream/").unwrap() + 8)..];
            let id: u64 = id_field[..(id_field.find(" ").unwrap())]
                .parse::<u64>()
                .unwrap();

            let item = match DB_MANAGER.lock().unwrap().get_item_direct(id) {
                Ok(value) => value,
                Err(_) => {
                    http::send_error(http::Status::NotFound404, &svr_cfg, stream);
                    return;
                }
            };

            http::send_file(
                &content,
                &item.file_path,
                stream,
                &svr_cfg,
                &item.get_mime_type(),
            );

            return;
        } else if content.find("/files/images/icon.png").is_some() {
            http::send_file(
                &content,
                "/var/lib/slms/icon.png",
                stream,
                &svr_cfg,
                "image/png",
            );

            return;
        }

        if xml.len() > 0 {
            let mut response =
                http::generate_header(xml.len(), "text/xml", false, &svr_cfg, http::Status::Ok200);

            response.push_str(&xml);

            con_manager.send_data(&response, stream);
        } else {
            con_manager.send_data(
                &http::generate_header(
                    0,
                    "text/html",
                    false,
                    &svr_cfg,
                    http::Status::InternalServerError500,
                ),
                stream,
            );
        }
    }

    fn print_welcome() {
        println!(
            "Simple Linux Media Server {}\nAuthor: Jörn Roddelkopf\n\nSee -h or --help for more Information\n",
            option_env!("CARGO_PKG_VERSION").unwrap_or("")
        );
    }

    fn print_helpscreen() {
        println!(
            "Commandline Options:\n\t-h\t--help\t\t\t\tDisplay Help Screen\n\t-c\t--configuration\t[PATH]\t\tUse the given Configuration File\n\t-d\t--dont-daemonize\t\tDo not run in Background\n",
        );
    }
}

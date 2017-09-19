use std::net::TcpListener;
use std::thread;
use std::sync::Mutex;
use daemonize::Daemonize;

use configuration::ConfigurationHandler;
use tools::Logger;
use database::DatabaseManager;
use server::SSDPServer;
use upnp::{ConnectionManager, ContentDirectory};
use provider::http;


lazy_static! { static ref db_manager: Mutex<DatabaseManager> = Mutex::new(DatabaseManager::new()); }

pub struct MediaServer {}

impl MediaServer {
    /// # TO-DO
    ///
    /// - Create Daemonizer for Background Process spawning
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
        let mut logger: Logger = Logger::new(
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

        // Bring up the Media Database
        db_manager.lock().unwrap().load(
            &cfg_handler.server_configuration.media_db_path,
            cfg_handler.server_configuration.share_dirs.clone(),
        );

        db_manager.lock().unwrap().boot_up();

        // Prepare the Sockets
        let listener = TcpListener::bind(("0.0.0.0", cfg_handler.server_configuration.server_port))
            .unwrap();

        // Bring up the SSDP Server
        let ssdp_server: SSDPServer = match SSDPServer::new(&cfg_handler.server_configuration) {
            Ok(value) => value,
            Err(_) => return,
        };

        match ssdp_server.discover() {
            Ok(_) => {}
            Err(_) => return,
        }

        println!("WAITING FOR CONNECTIONS");

        // Process Incoming Connections in new Threads
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let tcfg_handler = cfg_handler.clone();
            let svr_cfg = cfg_handler.server_configuration.clone();

            thread::spawn(move || {
                let mut response: String = String::new();
                let con_manager: ConnectionManager = ConnectionManager::new(&svr_cfg);
                let content: String = con_manager.handle_connection(&mut stream);
                let mut xml: String = String::new();

                if content.find("/connection/").is_some() {
                    xml = con_manager.handle_request(&content);
                } else if content.find("/content/").is_some() {
                    let db = db_manager.lock().unwrap();
                    let mut con_dir: ContentDirectory = ContentDirectory::new(&tcfg_handler, &db);
                    xml = con_dir.handle_request(&content);
                } else if content.find("/stream/").is_some() {
                    // Streaming
                    let id_field: &str = &content[(content.find("/stream/").unwrap() + 8)..];
                    let id: u64 = id_field[..(id_field.find(" ").unwrap())]
                        .parse::<u64>()
                        .unwrap();

                    let item = match db_manager.lock().unwrap().get_item_direct(id) {
                        Ok(value) => value,
                        Err(_) => {
                            http::send_error(http::Status::NotFound404, &svr_cfg, &mut stream);
                            return;
                        }
                    };

                    http::send_file(
                        &content,
                        &item.file_path,
                        &mut stream,
                        &svr_cfg,
                        &item.get_mime_type(),
                    );

                    return;
                } else if content.find("/files/images/icon.png").is_some() {
                    http::send_file(
                        &content,
                        "/var/lib/slms/icon.png",
                        &mut stream,
                        &svr_cfg,
                        "image/png",
                    );

                    return;
                }

                if xml.len() > 0 {
                    response = http::generate_header(
                        xml.len(),
                        "text/xml",
                        false,
                        &svr_cfg,
                        http::Status::Ok200,
                    );

                    response.push_str(&xml);

                    con_manager.send_data(&response, &mut stream);
                } else {
                    response = http::generate_header(
                        0,
                        "text/html",
                        false,
                        &svr_cfg,
                        http::Status::InternalServerError500,
                    );

                    con_manager.send_data(&response, &mut stream);
                }
            });
        }

        // Clean Up
        ssdp_server.byebye();
    }

    fn print_welcome() {
        println!(
            "Simple Linux Media Server\nAuthor: Jörn Roddelkopf\n\nSee -h or --help for more Information\n"
        );
    }

    fn print_helpscreen() {
        println!(
            "Commandline Options:\n\t-h\t--help\t\t\t\tDisplay Help Screen\n\t-c\t--configuration\t[PATH]\t\tUse the given Configuration File\n\t-d\t--dont-daemonize\t\tDo not run in Background\n",
        );
    }
}

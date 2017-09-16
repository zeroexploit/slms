/// # ServerConfiguration
///
/// This structures holdes the Servers Main Configuration
/// Attributes. As SLMS provides individual Configurations
/// per Renderer, these Options are the most basic ones,
/// affecting main Operation.
pub struct ServerConfiguration {
    pub server_name: String, // The Servers Name as it is displayed to the Network
    pub renderer_dir: String, // The Path to the Folder holding the Renderer Configurations
    pub default_renderer_path: String, // Path to the File that holds the Default Configuration for unknown devices
    pub thumbnail_dir: String, // Path to the Directory where Thumbnails should be stored
    pub server_port: u16, // Port to run the Server on
    pub server_interface: String, // Network Interface to run the Server on
    pub share_dirs: Vec<String>, // Pathes to the Folders that should be shared
    pub generate_thumbnails: bool, // Generate Thumbnails?
    pub log_path: String, // Path to the Log File
    pub log_level: u8, // Log Level to use
    pub server_tag: String, // Server Tag as Idenification
    pub server_ip: String, // Server IP
    pub server_uuid: String, // Server UUID
    pub media_db_path: String, // Path where to store the Media Database
}

impl ServerConfiguration {
    /// Creates a new Server Configuration Structure with the most basic
    /// Settings. Can be used in case the users Cfg is not available.
    pub fn new() -> ServerConfiguration {
        ServerConfiguration {
            server_name: String::from("SLMS"),
            renderer_dir: String::from("/etc/slms/renderer/"),
            default_renderer_path: String::from("/etc/slms/renderer/default.cfg"),
            thumbnail_dir: String::from("/var/lib/slms/thumbnails/"),
            server_port: 5001,
            server_interface: String::from("eth0"),
            share_dirs: Vec::new(),
            generate_thumbnails: false,
            log_path: String::from("/var/log/slms.log"),
            log_level: 0,
            server_tag: String::from("SLMS"),
            server_ip: String::from("127.0.0.1"),
            server_uuid: String::from("xxx"),
            media_db_path: String::from("/var/lib/slms/db.xml"),
        }
    }

    pub fn clone(&self) -> ServerConfiguration {
        ServerConfiguration {
            server_name: self.server_name.clone(),
            renderer_dir: self.renderer_dir.clone(),
            default_renderer_path: self.default_renderer_path.clone(),
            thumbnail_dir: self.thumbnail_dir.clone(),
            server_port: self.server_port,
            server_interface: self.server_interface.clone(),
            share_dirs: self.share_dirs.clone(),
            generate_thumbnails: self.generate_thumbnails,
            log_path: self.log_path.clone(),
            log_level: self.log_level,
            server_tag: self.server_tag.clone(),
            server_ip: self.server_ip.clone(),
            server_uuid: self.server_uuid.clone(),
            media_db_path: self.media_db_path.clone(),
        }
    }
}

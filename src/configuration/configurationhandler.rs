use std::io::BufReader;
use std::io::BufRead;
use std::fs::File;
use std::fs;
use super::serverconfiguration::ServerConfiguration;
use super::rendererconfiguration::RendererConfiguration;
use super::rendererconfiguration::SourceTargetMap;

/// # ConfigurationHandler
///
/// Stores, parses and manages the Media Servers Configuration
/// Files and provides the corresponding Structures to
/// parts of the Software where these are required.
///
/// # TO-DO
/// - If there is no Interface given -> Determine the Default Network Interface instead of setting it to eth0
/// - Determine what Path to use when no share Folder is given -> Security Risk to share whole File System
/// - Add Checks to determine if Configuration is set AND USEABLE / AVAILABLE -> Check Path / File Access, Interfaces, etc.
/// - Handle File System Errors -> Not Found, Permission Denied...
/// - Add check for Renderers Configuration
/// - Replace all Unwraps with error checks
/// - Determine the Servers Ip Address
/// - Generate Unique UUID
pub struct ConfigurationHandler {
    pub server_configuration: ServerConfiguration, // Server Configurations
    pub renderer_configurations: Vec<RendererConfiguration>, // List of Renderer Configurations
    pub default_index: usize, // Position of the Default Renderer inside the renderer_configurations List
    pub cfg_file_path: String, // Path to the Configuration File
}

impl ConfigurationHandler {
    /// Creates a new ConfigurationHandler ready to parse the
    /// Configuration Files.
    pub fn new() -> ConfigurationHandler {
        ConfigurationHandler {
            server_configuration: ServerConfiguration::new(),
            renderer_configurations: Vec::new(),
            default_index: 0,
            cfg_file_path: String::from("/etc/slms/server.cfg"),
        }
    }

    /// Read in the Servers Configuration from the File at
    /// the given Path. The server_configuration Structure
    /// will than hold the readed Configuration.
    /// If there was an Error this function returns false,
    /// and true if everything went well.
    pub fn parse(&mut self, cfg_path: &str) -> bool {
        self.cfg_file_path = cfg_path.to_string();
        let mut success = true;

        // Open Cfg File
        let cfg_file = File::open(&self.cfg_file_path).unwrap();
        let file = BufReader::new(&cfg_file);

        // Read all Lines
        for line in file.lines() {
            let line_res = line.unwrap();
            let mut name = line_res.clone();

            // Skip Comments
            if name.len() <= 1 || &name[0..1] == "#" || &name[0..1] == "\n" {
                continue;
            }

            name = name[..name.find("=").unwrap()].trim().to_string();
            let mut value = line_res;
            value = value[(value.find("=").unwrap() + 1)..].trim().to_string();

            if value.len() == 0 {
                continue;
            }

            println!("Name: \"{}\" - Value: \"{}\"", name, value);

            // Get Settings according their names
            match name.to_lowercase().as_ref() {
                "servername" => self.server_configuration.server_name = value.to_string(),
                "rendererdir" => self.server_configuration.renderer_dir = value.to_string(),
                "defaultrenderer" => {
                    self.server_configuration.default_renderer_path = value.to_string()
                }
                "thumbnaildir" => self.server_configuration.thumbnail_dir = value.to_string(),
                "serverport" => {
                    self.server_configuration.server_port = value.parse::<u16>().unwrap()
                }
                "serverinterface" => self.server_configuration.server_interface = value.to_string(),
                "folders" => {
                    self.server_configuration.share_dirs =
                        value.split(";").map(|s| s.to_string()).collect()
                }
                "generatethumbnails" => {
                    if value == "true" || value == "1" {
                        self.server_configuration.generate_thumbnails = true;
                    } else {
                        self.server_configuration.generate_thumbnails = false;
                    }
                } 
                "logfile" => self.server_configuration.log_path = value.to_string(),
                "loglevel" => self.server_configuration.log_level = value.parse::<u8>().unwrap(),
                _ => (),
            }
        }

        // Check the Configuration if everything was set and is usable / available -> Back to defaults if not
        if self.server_configuration.default_renderer_path.len() == 0 {
            self.server_configuration.default_renderer_path =
                String::from("/etc/slms/renderer/default.cfg");
            success = false;
        }

        if self.server_configuration.log_path.len() == 0 {
            self.server_configuration.log_path = String::from("/var/log/slms.log");
            success = false;
        }

        if self.server_configuration.renderer_dir.len() == 0 {
            self.server_configuration.default_renderer_path = String::from("/etc/slms/renderer/");
            success = false;
        }

        if self.server_configuration.server_interface.len() == 0 {
            self.server_configuration.server_interface = String::from("eth0"); // CHECK THIS TO DETERMIN REAL DEFAULT INTERFACE
            success = false;
        }

        if self.server_configuration.server_name.len() == 0 {
            self.server_configuration.server_name = String::from("SLMS");
            success = false;
        }

        if self.server_configuration.share_dirs.len() == 0 {
            self.server_configuration.share_dirs.push(String::from("/"));
            success = false;
        }

        if self.server_configuration.thumbnail_dir.len() == 0 {
            self.server_configuration.thumbnail_dir = String::from("/var/lib/slms/thumbnails/");
            success = false;
        }

        // Parse all Renderers
        println!("RENDERDIR: {}", self.server_configuration.renderer_dir);
        let paths = fs::read_dir(self.server_configuration.renderer_dir.clone()).unwrap();

        for path in paths {
            match path {
                Ok(element) => {
                    if element.path().is_file() {
                        if self.parse_renderer(element.path().to_str().unwrap()) == false {
                            success = false;
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }

        success
    }

    /// Parses a Renderers Configuration and adds it to the List of
    /// Renderers. Returns true if everything went well and false
    /// if something did not work.
    fn parse_renderer(&mut self, path: &str) -> bool {
        let success = true;
        let mut rnd_cfg: RendererConfiguration = RendererConfiguration::new();

        // Open Cfg File
        let cfg_file = File::open(path).unwrap();
        let file = BufReader::new(&cfg_file);

        // Read all Lines
        for line in file.lines() {
            let line_res = line.unwrap();
            let mut name = line_res.clone();

            // Skip Comments
            if name.len() <= 1 || &name[0..1] == "#" || &name[0..1] == "\n" {
                continue;
            }

            name = name[..name.find("=").unwrap()].trim().to_string();
            let mut value = line_res;
            value = value[(value.find("=").unwrap() + 1)..].trim().to_string();

            // Skip empty Values
            if value.len() == 0 {
                continue;
            }

            println!("Name: \"{}\" - Value: \"{}\"", name, value);

            match name.to_lowercase().as_ref() {
                "displayname" => rnd_cfg.display_name = value,
                "useragentsearchstring" => rnd_cfg.user_agent_search.push(value),
                "remoteipaddress" => rnd_cfg.remote_ip = value,
                "filextensions" => {
                    rnd_cfg.file_extensions = value.split(",").map(|s| s.to_string()).collect()
                }
                "conmap" => {
                    let mut tmp_con: SourceTargetMap = SourceTargetMap::new();
                    tmp_con.source = value[..value.find(":").unwrap()].to_string();
                    tmp_con.target = value[(value.find(":").unwrap() + 1)..].to_string();
                    rnd_cfg.container_maps.push(tmp_con);
                }
                "transcodecontainer" => rnd_cfg.transcode_container = value,
                "audiochannels" => rnd_cfg.audio_channels = value.parse::<u8>().unwrap(),
                "transcodeenabled" => {
                    if value == "true" || value == "1" {
                        rnd_cfg.transcode_enabled = true;
                    } else {
                        rnd_cfg.transcode_enabled = false;
                    }
                }
                "transcodeaudioenabled" => {
                    if value == "true" || value == "1" {
                        rnd_cfg.transcode_audio_enabled = true;
                    } else {
                        rnd_cfg.transcode_audio_enabled = false;
                    }
                }
                "transcodevideoenabled" => {
                    if value == "true" || value == "1" {
                        rnd_cfg.transcode_video_enabled = true;
                    } else {
                        rnd_cfg.transcode_video_enabled = false;
                    }
                }
                "transcodecodec" => {
                    let mut tmp_codec: SourceTargetMap = SourceTargetMap::new();
                    tmp_codec.source = value[..value.find(":").unwrap()].to_string();
                    tmp_codec.target = value[(value.find(":").unwrap() + 1)..].to_string();
                    rnd_cfg.transcode_codecs.push(tmp_codec);
                }
                "audiolanguage" => {
                    rnd_cfg.audio_languages = value.split(",").map(|s| s.to_string()).collect()
                }
                "subtitleconnection" => {
                    let mut tmp_sub: SourceTargetMap = SourceTargetMap::new();
                    tmp_sub.source = value[..value.find(":").unwrap()].to_string();
                    tmp_sub.target = value[(value.find(":").unwrap() + 1)..].to_string();
                    rnd_cfg.subtitle_connection.push(tmp_sub);
                }
                "encodesubtitles" => {
                    if value == "true" || value == "1" {
                        rnd_cfg.encode_subtitles = true;
                    } else {
                        rnd_cfg.encode_subtitles = false;
                    }
                }
                "titleinsteadofname" => {
                    if value == "true" || value == "1" {
                        rnd_cfg.title_instead_of_name = true;
                    } else {
                        rnd_cfg.title_instead_of_name = false;
                    }
                }
                "hidefileextension" => {
                    if value == "true" || value == "1" {
                        rnd_cfg.hide_file_extension = true;
                    } else {
                        rnd_cfg.hide_file_extension = false;
                    }
                }
                "muxtomatch" => {
                    if value == "true" || value == "1" {
                        rnd_cfg.mux_to_match = true;
                    } else {
                        rnd_cfg.mux_to_match = false;
                    }
                }
                _ => (),
            }
        }

        self.renderer_configurations.push(rnd_cfg);

        success
    }
}

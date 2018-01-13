use std::fs::File;
use std::io::{Write, Read};
use std::path::Path;
use std::fs;
use std::time;

use super::folder::Folder;
use media::{Item, Container, MediaType, Stream, StreamType, Thumbnail, mediaparser};
use tools::{NameValuePair, XMLParser, XMLEntry};


/// # DatabaseManager
///
/// This structure is one of the main parts of slms.
/// It holds all available Media Files and Folders,
/// it provides the connection beetween the Database
/// and the Filesystem as well as performs any Browse
/// and search requests on the existing Media Structure.
///
/// This is the place where all the information about Media
/// Files are stored and the Database is keept up to date
/// even if some Folders / Files are added / changed
/// while the System is running. Every action that requires
/// any type of Media provided by SLMS should go through
/// this structure.
///
/// # TO-DO
///
/// - Add Media Container Formats once FFMpeg can be compiled again
/// - Dynamically update the DB if Files / Folders on the FS have changed
/// - Allow execution even if no Database is available / XML Errors occured
pub struct DatabaseManager {
    path: String,
    media_item: Vec<Item>,
    media_folders: Vec<Folder>,
    share_folders: Vec<String>,
    media_formats: Vec<Container>,
    latest_id: u64,
}

impl DatabaseManager {
    /// This function inits the DatabaseManager Structure to hold
    /// the entire Media Database and also manages it. It is required to
    /// specify a path where the Database should be stored (as XML File)
    /// and the current media shares as well.
    /// In Order to actually use the DB a call to boot_up() is required
    /// after this initialization.
    ///
    /// # Arguments
    ///
    /// * `db_path` - Where to store the XML Database File
    /// * `shares` - List of Pathes to share theough SLMS
    pub fn load(&mut self, db_path: &str, shares: Vec<String>) {
        self.path = db_path.to_string();
        self.share_folders = shares;
        self.latest_id = 1;
    }
    /// This function creates a new DatabaseManager Structure that holds
    /// the entire Media Database and also manages it. The Path to store
    /// the Database is set to the default /var/lib/slms/db.xml Path.
    /// Should be used for initialisation only and be followed by
    /// a call to load(path, shares) in Order to make the DBManager
    /// usable.
    pub fn new() -> DatabaseManager {
        DatabaseManager {
            path: String::from("/var/lib/slms/db.xml"),
            media_item: Vec::new(),
            media_folders: Vec::new(),
            share_folders: Vec::new(),
            media_formats: Vec::new(),
            latest_id: 1,
        }
    }

    /// This function will boot up the Media Database.
    /// It will search for an existing XML File and if
    /// available parse its contents.
    /// Once the last Status was loaded, all Shares will
    /// be went through in Order to check if something
    /// new is available or some Files may have changed.
    /// Once everything has been parsed the new
    /// Database will be written back to the File System.
    ///
    /// As this function takes up a lot of time it is
    /// exlusivly designed to be called once when the
    /// Media Server starts up. If any changes happen while
    /// running, other functions will handle that cases.
    pub fn boot_up(&mut self) {

        // Load Database from File System
        self.load_database();

        info!("DB: All Items loaded. Negotiating with the File System...");

        // Parse all Shares and update Database
        for share in &self.share_folders.clone() {
            self.parse_folder(&share, 0);
        }

        info!("DB: Refreshed Database. Saving Changes...");

        // Store Database to File System
        self.save_database(true);

        // Ouput Information
        info!(
            "DB: Database Ready. There is a total of {} Folders and {} Files available.",
            self.media_folders.len(),
            self.media_item.len()
        );
    }

    /// Parses a Folder with the given Format. Calls the same function
    /// for all sub folders and tries to parse any File that is
    /// located inside the Folder.
    /// If the File is already present inside the Database, the
    /// File is skipped as long as it has not been modified.
    /// If a File is Unknown or was changed, the MediaParser
    /// is used to parse the File and add it to the Meida
    /// Database.
    ///
    /// Calling this function after the Database has been loaded
    /// from File and for all shares, this ensures that every
    /// Media laying under the shares will be part of the
    /// Media Database.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Folder that should be parsed
    /// * `parent_id` - Id of the Parent Folder this one lays in
    fn parse_folder(&mut self, path: &str, parent_id: u64) {
        info!("DB - parse_folder(): Parsing Folder: {}", path);

        // If the path does not exits -> return
        if path.is_empty() || DatabaseManager::does_exist(path) == false {
            warn!("DB - parse_folder(): Unable to parse Folder: {}", path);
            return;
        }

        let mut is_new: bool = false;
        let mut id: u64 = 0;

        // Check if that folder is in Database
        match self.get_folder_from_path(path) {
            Ok(folder) => {
                if DatabaseManager::get_last_modified(path) > folder.last_modified {
                    // If something changed update that folder
                    folder.last_modified = DatabaseManager::get_last_modified(path);
                    folder.element_count = DatabaseManager::get_elements(path);
                    id = folder.id;

                    debug!(
                        "DB - parse_folder(): Folder: {} was modified. Updated DB Entry...",
                        path
                    );
                }
            }
            Err(_) => {
                is_new = true;

                debug!(
                    "DB - parse_folder(): Folder: {} is new. Creating new DB Entry...",
                    path
                );
            }
        }

        // If not get information and add it to list
        if is_new {

            let mut folder = Folder::new();
            folder.id = self.get_next_id();
            folder.parent_id = parent_id;

            if &path[path.len() - 1..] == "/" {
                folder.title = match path[..path.len() - 1].split("/").last() {
                    Some(value) => value.to_string(),
                    None => {
                        error!("DB - parse_folder(): Unable to Split String: {}", path);
                        return;
                    }
                };
            } else {
                folder.title = match path.split("/").last() {
                    Some(value) => value.to_string(),
                    None => {
                        error!("DB - parse_folder(): Unable to Split String: {}", path);
                        return;
                    }
                };
            }

            // Skip Folders that are hidden
            if &folder.title[..1] == "." {
                debug!("DB - parse_folder(): Skipping hidden Folder: {}", path);
                return;
            }

            folder.path = path.to_string();
            folder.last_modified = DatabaseManager::get_last_modified(path);
            folder.element_count = DatabaseManager::get_elements(path);
            id = folder.id;
            self.media_folders.push(folder);
        }

        // Go through all Elements inside this Folder and add them
        let paths = match fs::read_dir(path) {
            Ok(value) => value,
            Err(e) => {
                warn!(
                    "DB - parse_folder(): Unable to access Elements in: {} - Reason: {}",
                    path,
                    e
                );
                return;
            }
        };

        for path in paths {
            match path {
                Ok(element) => {
                    let ele_path = element.path();
                    let ele_str = match ele_path.to_str() {
                        Some(value) => value,
                        None => {
                            error!("DB - parse_folder(): Unable to convert to str");
                            continue;
                        }
                    };

                    // If this is another folder -> parse it too
                    if element.path().is_dir() {
                        self.parse_folder(ele_str, id);
                    } else {
                        // If this is a file -> use the media parser
                        let mut is_new = false;

                        // Check if already existing
                        match self.get_item_from_path(ele_str) {
                            Ok(some) => {
                                // Check if something changed
                                if DatabaseManager::get_last_modified(ele_str) >
                                    some.last_modified
                                {
                                    debug!(
                                        "DB - parse_folder(): File: {} was modified. Update DB Entry...",
                                        ele_str
                                    );
                                    // Reparse the item
                                    if !mediaparser::parse_file(ele_str, some) {
                                        warn!(
                                            "DB - parse_folder(): Unable to parse and update File: {} !",
                                            ele_str
                                        );
                                    }
                                }
                            }
                            Err(_) => {
                                is_new = true;
                                debug!(
                                    "DB - parse_folder(): Found new File: {}. Creating new DB Entry...",
                                    ele_str
                                );
                            }
                        }

                        // Parse if new and assign Ids
                        if is_new {
                            let mut item: Item = Item::new();
                            if mediaparser::parse_file(ele_str, &mut item) {
                                item.id = self.get_next_id();

                                item.parent_id = id;
                                // Skip hidden Files
                                if item.meta_data.file_name.len() > 0 {
                                    if &item.meta_data.file_name[..1] == "." {
                                        debug!(
                                            "DB - load_database(): Skipping hidden File: {}",
                                            ele_str
                                        );

                                    } else {
                                        self.media_item.push(item);
                                    }
                                } else {
                                    error!(
                                        "DB - load_database(): Unable to determine Filename for: {}",
                                        ele_str
                                    );
                                }
                            } else {
                                warn!("DB - load_database(): Unable to parse File: {}", ele_str);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "DB - load_database(): Unable to access Sub Elements! - Reason: {}",
                        e
                    );
                    continue;
                }
            }
        }
    }

    /// Checks if a Folder at the given Path exists inside
    /// the Database and returns it if available or causes
    /// Err if not available.
    /// This is important in Order to check if a given Folder
    /// is already part of the Media Database.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Folder to check
    fn get_folder_from_path(&mut self, path: &str) -> Result<&mut Folder, ()> {
        for folder in &mut self.media_folders {
            if folder.path == path {
                return Ok(folder);
            }
        }

        Err(())
    }

    /// Checks if a Item with the given Path already exits
    /// inside the Database and returns it if available.
    /// Err if not.
    /// This is important in Order to check if a given Item
    /// is already part of the Media Database.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Item to check
    fn get_item_from_path(&mut self, path: &str) -> Result<&mut Item, ()> {
        for item in &mut self.media_item {
            if item.file_path == path {
                return Ok(item);
            }
        }
        Err(())
    }

    /// Returns the Number of Elements inside the given
    /// Path. If something went wrong, 0 is returned.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Folder to count Elements in
    fn get_elements(path: &str) -> u32 {
        let paths = match fs::read_dir(path) {
            Ok(value) => value,
            Err(_) => return 0,
        };
        let mut counter = 0;

        for path in paths {
            if path.is_ok() {
                counter += 1;
            }
        }

        counter
    }

    /// Stores the current Media Database into an XML File.
    /// The DatabaseManagers.path Variable will be used
    /// as storage location.
    ///
    /// If it is not possible to write a XML File, no
    /// Database will be available on next boot. That might
    /// dramatically slow down browsing operations!
    ///
    /// While storing the Database a consitency check might
    /// be performed in order to avoid saving non
    /// existing or changed content. If the check is activated
    /// it might decrease performance and should therefore
    /// only be done in non crictial situation. E.g.:
    /// Not while a user is browsing the content.
    ///
    /// # Arguments
    ///
    /// * `check_changed` - Do or do not check if Files / Folders has changed since last save
    fn save_database(&self, check_changed: bool) {
        // Create ROOT Folder
        let mut root_attr: Vec<NameValuePair> = Vec::new();
        root_attr.push(NameValuePair::new("id", "0"));
        root_attr.push(NameValuePair::new("parentId", "-1"));
        root_attr.push(NameValuePair::new("title", "root"));
        root_attr.push(NameValuePair::new("path", ""));
        root_attr.push(NameValuePair::new(
            "count",
            &self.share_folders.len().to_string(),
        ));
        root_attr.push(NameValuePair::new("last_modified", "0"));

        // Start XML
        let mut xml_parser: XMLParser = XMLParser::new();
        xml_parser.start_xml();

        // Insert root
        xml_parser.open_tag("root", &root_attr, true);

        // Insert all Folders
        for folder in &self.media_folders {
            // Check if folder exists -> skip everything else if not -> there can not be any content if the parent is lost
            if check_changed {
                if DatabaseManager::does_exist(&folder.path) == false {
                    continue;
                }
            }

            xml_parser.open_tag("folder", &folder.get_name_value_pairs(), false);
        }

        // Go through all Items
        for item in &self.media_item {
            // Check if file exists and was not changed
            if check_changed {
                if DatabaseManager::does_exist(&item.file_path) {
                    // Add a File only if nothing has change
                    if DatabaseManager::get_last_modified(&item.file_path) > item.last_modified {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            xml_parser.open_tag("item", &item.get_name_value_pairs(), true);

            // Write Streams
            for stream in &item.media_tracks {
                xml_parser.open_tag("stream", &stream.get_name_value_pairs(), false);
            }

            // Write Thumbnail Data if available
            if item.thumbnail.is_available() {
                xml_parser.open_tag("thumbnail", &item.thumbnail.get_name_value_pairs(), false);
            }

            // Write Meta Tags
            let meta_attr = item.meta_data.get_name_value_pairs();

            for meta in meta_attr {
                let tmp_list: Vec<NameValuePair> = vec![
                    NameValuePair::new("name", &meta.name),
                    NameValuePair::new("value", &meta.value),
                ];
                xml_parser.open_tag("meta", &tmp_list, false);
            }

            xml_parser.close_tag("item");

        }

        xml_parser.close_tag("root");

        // Insert Container Formats
        for container in &self.media_formats {
            xml_parser.open_tag("format", &container.get_name_value_pairs(), true);

            for extension in &container.file_extensions {
                let tmp_list: Vec<NameValuePair> = vec![NameValuePair::new("value", &extension)];
                xml_parser.open_tag("extension", &tmp_list, false);
            }

            for mime in &container.mime_types {
                let tmp_list: Vec<NameValuePair> = vec![NameValuePair::new("value", &mime)];
                xml_parser.open_tag("mime", &tmp_list, false);
            }

            xml_parser.close_tag("format");
        }

        let mut db_file = match File::create(&self.path) {
            Ok(some) => some,
            Err(e) => {
                error!(
                    "DB - load_database(): Unable to create DB File: {} - Reason: {}",
                    self.path,
                    e
                );
                return;
            }
        };

        match db_file.write(xml_parser.xml_content.as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                error!(
                    "DB - load_database(): Unable to write DB File: {} - Reason: {}",
                    self.path,
                    e
                );
            }
        }
    }

    /// Opens the XML File containing the Media Database and parses it into the
    /// corresponding attributes in DatabaseManager Structure. The DatabaseManager.path
    /// variable is used to find the XML File. Make sure it is set correctly.
    ///
    /// # No XML File available
    /// If no XML File is found, can be accessed or there is any other reason it is not
    /// available, nothing will be loaded and the Media Database stays empty! If it is
    /// the first time the slms runs, this is normal. In any other situation this
    /// has a great negative impact on the browsing performance!
    ///
    /// # File Check
    /// While reading the XML Entries two checks are performed:
    /// * Does a File and/or Folder still exists?
    /// * Was a File and/or Folder not modified?
    /// If one of these is false, the corresponding element will be skipped and not
    /// be loaded. In case of a Folder, any furhter child content will be skipped
    /// if it no longer exists, because there can not be any childs without the
    /// parent element. In case a Folder exists but was changed the folder
    /// will be skipped but not its contents as there might be some minor changes
    /// like added files, but everything else might still remain unchanged. This
    /// might save some amount of Media-File-Parsing-Time, which has a great
    /// impact on the start up time of slms.
    /// The reason for this is that any change means a File / Folder needs to
    /// be parsed anyway and we already parse everything not already in list.
    /// Keeping changes out of the list avoids the need to do a sperate check if
    /// something has changed on boot up time.
    fn load_database(&mut self) {
        // Open Database File
        debug!("DB - load_database(): Open Database File: {}", self.path);
        let mut db_file = match File::open(&self.path) {
            Ok(some) => some,
            Err(e) => {
                warn!(
                    "DB - load_database(): Unable to Open Database File: {} - Reason: {}",
                    self.path,
                    e
                );
                return;
            }
        };

        debug!("DB - load_database(): Reading Database File Content");

        let mut contents = String::new();
        match db_file.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(e) => {
                warn!(
                    "DB - load_database(): Unable to read from Database File: {} - Reason: {}",
                    self.path,
                    e
                );
                return;
            }
        }

        debug!("DB - load_database(): Content loaded. Parsing XML...");

        let xml_parser: XMLParser = XMLParser::open(&contents);
        let mut root_xml: XMLEntry = XMLEntry::new();

        debug!("DB - load_database(): XML loaded. Moving Root and Formats to Memory...");


        // Read Root and Format Tags
        for entry in xml_parser.xml_entries {
            match entry.tag.as_ref() {
                "root" => {
                    root_xml = entry;
                }
                "format" => {
                    let mut media_container: Container = Container::new();
                    media_container.id =
                        match XMLParser::get_value_from_name(&entry.attributes, "id")
                            .parse::<u64>() {
                            Ok(value) => value,
                            Err(_) => {
                                warn!(
                        "DB - load_database(): Unable to parse Format Id...",
                    );
                                continue;
                            }
                        };
                    media_container.name =
                        XMLParser::get_value_from_name(&entry.attributes, "name");

                    for sub_tag in entry.sub_tags {
                        match sub_tag.tag.as_ref() {
                            "extension" => {
                                media_container.file_extensions.push(
                                    XMLParser::get_value_from_name(
                                        &sub_tag.attributes,
                                        "value",
                                    ),
                                )
                            }
                            "mime" => {
                                media_container.mime_types.push(XMLParser::get_value_from_name(
                                    &sub_tag.attributes,
                                    "value",
                                ))
                            }
                            _ => (),
                        }
                    }

                    self.media_formats.push(media_container);
                }

                _ => (),
            }
        }

        // Media Folders
        for folder in root_xml.sub_tags {
            if folder.tag == "folder" {
                let mut tmp_folder: Folder = Folder::new();
                tmp_folder.element_count =
                    match XMLParser::get_value_from_name(&folder.attributes, "count")
                        .parse::<u32>() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!(
                        "DB - load_database(): Unable to parse Folder Element Count...",
                    );
                            continue;
                        }
                    };
                tmp_folder.id = match XMLParser::get_value_from_name(&folder.attributes, "id")
                    .parse::<u64>() {
                    Ok(value) => value,
                    Err(_) => {
                        warn!(
                        "DB - load_database(): Unable to parse Folder Id...",
                    );
                        continue;
                    }
                };
                tmp_folder.parent_id =
                    match XMLParser::get_value_from_name(&folder.attributes, "parentId")
                        .parse::<u64>() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!(
                        "DB - load_database(): Unable to parse Folder Parent Id...",
                    );
                            continue;
                        }
                    };
                tmp_folder.last_modified =
                    match XMLParser::get_value_from_name(&folder.attributes, "lastModified")
                        .parse::<u64>() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!(
                        "DB - load_database(): Unable to parse Folder last modified Date..",
                    );
                            continue;
                        }
                    };
                tmp_folder.path = XMLParser::get_value_from_name(&folder.attributes, "path");
                tmp_folder.title = XMLParser::get_value_from_name(&folder.attributes, "title");

                self.set_latest_id(tmp_folder.id);

                // Check if folder exists -> skip everything else if not -> there can not be any content if the parent is lost
                if DatabaseManager::does_exist(&tmp_folder.path) {
                    // Add a Folder only if nothing has changed but still parse its contents as long as it exists
                    if DatabaseManager::get_last_modified(&tmp_folder.path) <=
                        tmp_folder.last_modified
                    {
                        debug!(
                        "DB - load_database(): Folder: {} has not changed...",
                        tmp_folder.path,
                    );
                        self.media_folders.push(tmp_folder);
                    } else {
                        debug!(
                        "DB - load_database(): Folder: {} has changed. Prepare to re-parse...",
                        tmp_folder.path,
                    );
                        self.media_folders.push(tmp_folder);
                    }
                } else {
                    debug!(
                        "DB - load_database(): Folder: {} does not exist any longer. Remove from DB...",
                        tmp_folder.path,
                    );
                    continue;
                }
            } else if folder.tag == "item" {
                // Media Items
                let mut tmp_item: Item = Item::new();
                tmp_item.duration = XMLParser::get_value_from_name(&folder.attributes, "duration");
                tmp_item.file_path = XMLParser::get_value_from_name(&folder.attributes, "path");
                tmp_item.file_size =
                    match XMLParser::get_value_from_name(&folder.attributes, "size")
                        .parse::<u64>() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!(
                        "DB - load_database(): Unable to parse Item Size...",
                    );
                            continue;
                        }
                    };
                tmp_item.id = match XMLParser::get_value_from_name(&folder.attributes, "id")
                    .parse::<u64>() {
                    Ok(value) => value,
                    Err(_) => {
                        warn!(
                        "DB - load_database(): Unable to parse Item Id...",
                    );
                        continue;
                    }
                };
                tmp_item.last_modified =
                    match XMLParser::get_value_from_name(&folder.attributes, "lastModified")
                        .parse::<u64>() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!(
                        "DB - load_database(): Unable to parse Item last modified Date...",
                    );
                            continue;
                        }
                    };
                tmp_item.media_type = MediaType::from_string(
                    &XMLParser::get_value_from_name(&folder.attributes, "type"),
                );
                tmp_item.parent_id =
                    match XMLParser::get_value_from_name(&folder.attributes, "parentId")
                        .parse::<u64>() {
                        Ok(value) => value,
                        Err(_) => {
                            warn!(
                        "DB - load_database(): Unable to parse Item Parent Id...",
                    );
                            continue;
                        }
                    };
                self.set_latest_id(tmp_item.id);

                // Streams
                for stream in folder.sub_tags {
                    match stream.tag.as_ref() {
                        "stream" => {
                            let mut tmp_stream: Stream = Stream::new();
                            tmp_stream.audio_channels = match XMLParser::get_value_from_name(
                                &stream.attributes,
                                "nrAudioChannels",
                            ).parse::<u8>() {
                                Ok(value) => value,
                                Err(_) => {
                                    warn!(
                        "DB - load_database(): Unable to parse Audio Channel Number...",
                    );
                                    continue;
                                }
                            };
                            tmp_stream.bit_depth = match XMLParser::get_value_from_name(
                                &stream.attributes,
                                "bitDepth",
                            ).parse::<u8>() {
                                Ok(value) => value,
                                Err(_) => {
                                    warn!(
                        "DB - load_database(): Unable to parse Stream Bit Depth...",
                    );
                                    continue;
                                }
                            };
                            tmp_stream.bitrate = match XMLParser::get_value_from_name(
                                &stream.attributes,
                                "bitrate",
                            ).parse::<u64>() {
                                Ok(value) => value,
                                Err(_) => {
                                    warn!(
                        "DB - load_database(): Unable to parse Stream Bit Rate...",
                    );
                                    continue;
                                }
                            };
                            tmp_stream.codec_name =
                                XMLParser::get_value_from_name(&stream.attributes, "codecName");
                            tmp_stream.index =
                                match XMLParser::get_value_from_name(&stream.attributes, "index")
                                    .parse::<u8>() {
                                    Ok(value) => value,
                                    Err(_) => {
                                        warn!(
                        "DB - load_database(): Unable to parse Stream Index...",
                    );
                                        continue;
                                    }
                                };
                            tmp_stream.is_default = match XMLParser::get_value_from_name(
                                &stream.attributes,
                                "isDefault",
                            ).parse::<bool>() {
                                Ok(value) => value,
                                Err(_) => {
                                    warn!(
                        "DB - load_database(): Unable to parse Stream is_default...",
                    );
                                    continue;
                                }
                            };
                            tmp_stream.is_forced = match XMLParser::get_value_from_name(
                                &stream.attributes,
                                "isForced",
                            ).parse::<bool>() {
                                Ok(value) => value,
                                Err(_) => {
                                    warn!(
                        "DB - load_database(): Unable to parse Stream is_forced...",
                    );
                                    continue;
                                }
                            };
                            tmp_stream.language =
                                XMLParser::get_value_from_name(&stream.attributes, "language");
                            tmp_stream.frame_width =
                                match XMLParser::get_value_from_name(&stream.attributes, "width")
                                    .parse::<u16>() {
                                    Ok(value) => value,
                                    Err(_) => {
                                        warn!(
                        "DB - load_database(): Unable to parse Stream width...",
                    );
                                        continue;
                                    }
                                };
                            tmp_stream.frame_height = match XMLParser::get_value_from_name(
                                &stream.attributes,
                                "height",
                            ).parse::<u16>() {
                                Ok(value) => value,
                                Err(_) => {
                                    warn!(
                        "DB - load_database(): Unable to parse Stream height...",
                    );
                                    continue;
                                }
                            };
                            tmp_stream.sample_rate = match XMLParser::get_value_from_name(
                                &stream.attributes,
                                "sampleFrequenzy",
                            ).parse::<u32>() {
                                Ok(value) => value,
                                Err(_) => {
                                    warn!(
                        "DB - load_database(): Unable to parse Stream Frequenzy...",
                    );
                                    continue;
                                }
                            };
                            tmp_stream.stream_type =
                                StreamType::from_string(
                                    &XMLParser::get_value_from_name(&stream.attributes, "type"),
                                );

                            tmp_item.media_tracks.push(tmp_stream);
                        }
                        "thumbnail" => {
                            let mut tmp_thumb: Thumbnail = Thumbnail::new();
                            tmp_thumb.file_path =
                                XMLParser::get_value_from_name(&stream.attributes, "path");
                            tmp_thumb.file_size =
                                match XMLParser::get_value_from_name(&stream.attributes, "size")
                                    .parse::<u64>() {
                                    Ok(value) => value,
                                    Err(_) => {
                                        warn!(
                        "DB - load_database(): Unable to parse Thumbnail Size...",
                    );
                                        continue;
                                    }
                                };
                            tmp_thumb.height = match XMLParser::get_value_from_name(
                                &stream.attributes,
                                "height",
                            ).parse::<u16>() {
                                Ok(value) => value,
                                Err(_) => {
                                    warn!(
                        "DB - load_database(): Unable to parse Thumbnail Height...",
                    );
                                    continue;
                                }
                            };
                            tmp_thumb.width =
                                match XMLParser::get_value_from_name(&stream.attributes, "width")
                                    .parse::<u16>() {
                                    Ok(value) => value,
                                    Err(_) => {
                                        warn!(
                        "DB - load_database(): Unable to parse Thumbnail Width...",
                    );
                                        continue;
                                    }
                                };
                            tmp_thumb.item_id = match XMLParser::get_value_from_name(
                                &stream.attributes,
                                "itemId",
                            ).parse::<u64>() {
                                Ok(value) => value,
                                Err(_) => {
                                    warn!(
                        "DB - load_database(): Unable to parse Thumbnail Item Id...",
                    );
                                    continue;
                                }
                            };
                            tmp_thumb.mime_type =
                                XMLParser::get_value_from_name(&stream.attributes, "mimeType");

                            tmp_item.thumbnail = tmp_thumb;
                        }
                        "meta" => {
                            tmp_item.insert_meta_data(
                                &XMLParser::get_value_from_name(&stream.attributes, "name"),
                                &XMLParser::get_value_from_name(&stream.attributes, "value"),
                            );
                        }
                        _ => (),
                    }
                }

                // Check if File still exists
                if DatabaseManager::does_exist(&tmp_item.file_path) {
                    debug!(
                        "DB - load_database(): File {} does still exist...",
                        tmp_item.file_path
                    );
                    self.media_item.push(tmp_item);
                } else {
                    debug!(
                        "DB - load_database(): File {} does not exist anymore. Ignoring...",
                        tmp_item.file_path
                    );
                }
            }
        }

        debug!("DB - load_database(): All Data loaded into Memory.");
    }

    /// Takes the given ID and sets it to be the highest one
    /// if it is greater than the last one.
    ///
    /// This is used to make sure the DatabaseManager always nows
    /// the last ID and is able to offer a new ID that is always
    /// unique over all Elements.
    ///
    /// # Arguments
    ///
    /// * `id` - Id to set
    fn set_latest_id(&mut self, id: u64) {
        if id > self.latest_id {
            self.latest_id = id;
        }
    }

    /// Returns the next free ID and automatically increases the
    /// latest_id value.
    fn get_next_id(&mut self) -> u64 {
        let id = self.latest_id;
        self.latest_id += 1;

        id
    }

    /// Returns the last modified date of the element with
    /// the given Path. The Value will be a UNIX Timestamp
    /// in seconds. Folders and Files are possible.
    /// Returns 0 if something went wrong. Attention: This
    /// will indicate an element will always be handled as
    /// modified!
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the File or Folder to get the last modified Date from
    fn get_last_modified(path: &str) -> u64 {
        let metadata = fs::metadata(path);
        match metadata {
            Ok(value) => {
                match value.modified() {
                    Ok(i_value) => {
                        match i_value.duration_since(time::UNIX_EPOCH) {
                            Ok(in_value) => in_value.as_secs(),
                            Err(_) => 0,
                        }
                    }
                    Err(_) => 0,
                }
            }
            Err(_) => 0,
        }
    }

    /// Checks if a File or Folder does exist. Returns
    /// true if so and false if not.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the File or Folder to check the Existance
    fn does_exist(path: &str) -> bool {
        let path = Path::new(path);
        if path.exists() { true } else { false }
    }

    /// Returns the List of Folders that got the given Id as Parent Folder.
    /// If the Folder was changed it will be updated!
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Id of the Element to get the Child-Folders for
    pub fn get_folder_from_parent(&mut self, parent_id: u64) -> Vec<Folder> {
        let mut res_vec: Vec<Folder> = Vec::new();

        // Find parent Folder and check if something changed
        for index in 0..self.media_folders.len() {
            if self.media_folders[index].id == parent_id {
                // Re-Parse folder if modified
                if self.media_folders[index].last_modified <
                    DatabaseManager::get_last_modified(&self.media_folders[index].path)
                {
                    let path = self.media_folders[index].path.clone();
                    self.parse_folder(&path, parent_id);
                }

                break;
            }
        }

        // Find all Folders with the given Parent ID
        for folder in &self.media_folders {
            if folder.parent_id == parent_id {
                res_vec.push(folder.clone());
            }
        }

        res_vec
    }

    /// Returns a List of Items that got the given Id as Parent Folder.
    /// If one of the Items has changed, it will be reparsed!
    ///
    /// # Arguments
    ///
    /// * `parent_id` - Id of the Element to get the Child-Item for
    pub fn get_items_from_parent(&mut self, parent_id: u64) -> Vec<Item> {
        let mut res_vec: Vec<Item> = Vec::new();

        for mut item in &mut self.media_item {
            if item.parent_id == parent_id {
                // Check if Item has changed
                if item.last_modified < DatabaseManager::get_last_modified(&item.file_path) {
                    // If the Item can not be parsed: Skip it
                    if !DatabaseManager::does_exist(&item.file_path) ||
                        !mediaparser::parse_file(&item.file_path.clone(), &mut item)
                    {
                        continue;
                    }
                }

                res_vec.push(item.clone());
            }
        }

        return res_vec;
    }

    /// Directly returns the Folder with the given Id
    ///
    /// # Arguments
    ///
    /// * `id` - Id of the Folder to get the Values for
    pub fn get_folder_direct(&self, id: u64) -> Result<Folder, ()> {
        for folder in &self.media_folders {
            if folder.id == id {
                return Ok(folder.clone());
            }
        }

        Err(())
    }

    /// Directly returns the Item with the given Id
    ///
    /// # Arguments
    ///
    /// * `id` - Id of the Item to get the Values for
    pub fn get_item_direct(&self, id: u64) -> Result<Item, ()> {
        for item in &self.media_item {
            if item.id == id {
                return Ok(item.clone());
            }
        }

        Err(())
    }
}

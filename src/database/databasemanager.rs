use media::Item;
use media::Container;
use media::MediaParser;
use media::MediaType;
use media::Stream;
use media::StreamType;
use media::Thumbnail;
use super::folder::Folder;
use tools::NameValuePair;
use tools::XMLParser;
use tools::XMLEntry;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::path::Path;
use std::fs;
use std::time;

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
pub struct DatabaseManager {
    path: String,
    media_item: Vec<Item>,
    media_folders: Vec<Folder>,
    share_folders: Vec<String>,
    media_formats: Vec<Container>,
    latest_id: u64,
    parser: MediaParser,
}

impl DatabaseManager {
    /// This function creates a new DatabaseManager Structure that holds
    /// the entire Media Database and also manages it. It is required to
    /// specifie a path where the Database should be stored (as XML File)
    /// and the current media shares as well.
    pub fn new(db_path: &str, shares: Vec<String>) -> DatabaseManager {
        DatabaseManager {
            path: db_path.to_string(),
            media_item: Vec::new(),
            media_folders: Vec::new(),
            share_folders: shares,
            media_formats: Vec::new(),
            latest_id: 1,
            parser: MediaParser::new(),
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
        println!("Opening Database...");
        self.load_database();

        // Parse all Shares and update Database
        println!("Parsing shares...");
        for share in self.share_folders.clone() {
            self.parse_folder(&share, 0);
        }

        // Store Database to File System
        println!("Storing Database...");
        self.save_database(true);
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
    fn parse_folder(&mut self, path: &str, parent_id: u64) {
        // If the path does not exits -> return
        if self.does_exist(path) == false {
            return ();
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
                }
            }
            Err(_) => {
                is_new = true;
            }
        }

        // If not get information and add it to list
        if is_new {

            let mut folder = Folder::new();
            folder.id = self.get_next_id();
            folder.parent_id = parent_id;
            if &path[path.len() - 1..] == "/" {
                folder.title = path[..path.len() - 1]
                    .split("/")
                    .last()
                    .unwrap()
                    .to_string();
            } else {
                folder.title = path.split("/").last().unwrap().to_string();
            }
            println!("New Folder: {}", folder.title);
            folder.path = path.to_string();
            folder.last_modified = DatabaseManager::get_last_modified(path);
            folder.element_count = DatabaseManager::get_elements(path);
            id = folder.id;
            self.media_folders.push(folder);
        }

        // Go through all Elements inside this Folder and add them
        let paths = fs::read_dir(path).unwrap();

        for path in paths {
            match path {
                Ok(element) => {
                    // If this is another folder -> parse it too
                    if element.path().is_dir() {
                        self.parse_folder(element.path().to_str().unwrap(), id);
                    } else {
                        // If this is a file -> use the media parser
                        let mut is_new = false;

                        // Check if already existing
                        match self.get_item_from_path(element.path().to_str().unwrap()) {
                            Ok(some) => {
                                // Check if something changed
                                if DatabaseManager::get_last_modified(
                                    element.path().to_str().unwrap(),
                                ) > some.last_modified
                                {
                                    is_new = true;
                                }
                            }
                            Err(_) => {
                                is_new = true;
                            }
                        }

                        // Parse if something changed -> skip if not -> why parse if we know everything?
                        if is_new {
                            let mut item: Item = Item::new();
                            if self.parser.parse_file(
                                element.path().to_str().unwrap(),
                                &mut item,
                            )
                            {
                                item.id = self.get_next_id();
                                item.parent_id = id;

                                println!("Parsed File: {}", element.path().to_str().unwrap());

                                self.media_item.push(item);
                            } else {
                                println!("Unable to parse: {}", element.path().to_str().unwrap());
                            }
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }
        }
    }

    /// Checks if a Folder at the given Path exists inside
    /// the Database and returns it if available or causes
    /// Err if not available.
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
    fn get_item_from_path(&mut self, path: &str) -> Result<&mut Item, ()> {
        for item in &mut self.media_item {
            if item.file_path == path {
                return Ok(item);
            }
        }
        Err(())
    }

    /// Returns the Number of Elements inside the given
    /// Path.
    fn get_elements(path: &str) -> u32 {
        let paths = fs::read_dir(path).unwrap();
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
                if self.does_exist(&folder.path) == false {
                    continue;
                }
            }

            xml_parser.open_tag("folder", &folder.get_name_value_pairs(), true);

            // Go through all Items
            for item in &self.media_item {
                // Write only the Items inside current Foder
                if item.parent_id == folder.id {

                    // Check if file exists and was not changed
                    if check_changed {
                        if self.does_exist(&item.file_path) {
                            // Add a File only if nothing has change
                            if DatabaseManager::get_last_modified(&item.file_path) >
                                item.last_modified
                            {
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
                        xml_parser.open_tag(
                            "thumbnail",
                            &item.thumbnail.get_name_value_pairs(),
                            false,
                        );
                    }

                    // Write Meta Tags
                    let meta_attr = item.meta_data.get_name_value_pairs();

                    for meta in meta_attr {
                        let tmp_list: Vec<NameValuePair> =
                            vec![
                                NameValuePair::new("name", &meta.name),
                                NameValuePair::new("value", &meta.value),
                            ];
                        xml_parser.open_tag("meta", &tmp_list, false);
                    }

                    xml_parser.close_tag("item");
                }
            }

            xml_parser.close_tag("folder");
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
            Err(_) => return (),
        };

        db_file.write(xml_parser.xml_content.as_bytes()).expect(
            "Unable to write to Database!",
        );
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
        let mut db_file = match File::open(&self.path) {
            Ok(some) => some,
            Err(_) => return (),
        };
        let mut contents = String::new();
        match db_file.read_to_string(&mut contents) {
            Ok(_) => println!("Openend Database..."),
            Err(_) => return (),
        }

        let xml_parser: XMLParser = XMLParser::open(&contents);
        let mut root_xml: XMLEntry = XMLEntry::new();

        // Read Root and Format Tags
        for entry in xml_parser.xml_entries {
            match entry.tag.as_ref() {
                "root" => root_xml = entry,
                "format" => {
                    let mut media_container: Container = Container::new();
                    media_container.id = XMLParser::get_value_from_name(&entry.attributes, "id")
                        .parse::<u64>()
                        .unwrap();
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
            let mut tmp_folder: Folder = Folder::new();
            tmp_folder.element_count = XMLParser::get_value_from_name(&folder.attributes, "count")
                .parse::<u32>()
                .unwrap();
            tmp_folder.id = XMLParser::get_value_from_name(&folder.attributes, "id")
                .parse::<u64>()
                .unwrap();
            tmp_folder.parent_id = XMLParser::get_value_from_name(&folder.attributes, "parentId")
                .parse::<u64>()
                .unwrap();
            tmp_folder.last_modified =
                XMLParser::get_value_from_name(&folder.attributes, "lastModified")
                    .parse::<u64>()
                    .unwrap();
            tmp_folder.path = XMLParser::get_value_from_name(&folder.attributes, "path");
            tmp_folder.title = XMLParser::get_value_from_name(&folder.attributes, "title");

            self.set_latest_id(tmp_folder.id);

            // Check if folder exists -> skip everything else if not -> there can not be any content if the parent is lost
            if self.does_exist(&tmp_folder.path) {
                // Add a Folder only if nothing has changed but still parse its contents as long as it exists
                if DatabaseManager::get_last_modified(&tmp_folder.path) <=
                    tmp_folder.last_modified
                {
                    self.media_folders.push(tmp_folder);
                }
            } else {
                continue;
            }

            // Media Items
            for item in folder.sub_tags {
                let mut tmp_item: Item = Item::new();
                tmp_item.duration = XMLParser::get_value_from_name(&item.attributes, "duration");
                tmp_item.file_path = XMLParser::get_value_from_name(&item.attributes, "path");
                tmp_item.file_size = XMLParser::get_value_from_name(&item.attributes, "size")
                    .parse::<u64>()
                    .unwrap();
                tmp_item.id = XMLParser::get_value_from_name(&item.attributes, "id")
                    .parse::<u64>()
                    .unwrap();
                tmp_item.last_modified =
                    XMLParser::get_value_from_name(&item.attributes, "lastModified")
                        .parse::<u64>()
                        .unwrap();
                tmp_item.media_type = MediaType::from_string(
                    &XMLParser::get_value_from_name(&item.attributes, "type"),
                );
                tmp_item.parent_id = XMLParser::get_value_from_name(&item.attributes, "parentId")
                    .parse::<u64>()
                    .unwrap();
                self.set_latest_id(tmp_item.id);

                // Streams
                for stream in item.sub_tags {
                    match stream.tag.as_ref() {
                        "stream" => {
                            let mut tmp_stream: Stream = Stream::new();
                            tmp_stream.audio_channels = XMLParser::get_value_from_name(
                                &stream.attributes,
                                "nrAudioChannels",
                            ).parse::<u8>()
                                .unwrap();
                            tmp_stream.bit_depth =
                                XMLParser::get_value_from_name(&stream.attributes, "bitDepth")
                                    .parse::<u8>()
                                    .unwrap();
                            tmp_stream.bitrate =
                                XMLParser::get_value_from_name(&stream.attributes, "bitrate")
                                    .parse::<u64>()
                                    .unwrap();
                            tmp_stream.codec_name =
                                XMLParser::get_value_from_name(&stream.attributes, "codecName");
                            tmp_stream.index =
                                XMLParser::get_value_from_name(&stream.attributes, "index")
                                    .parse::<u8>()
                                    .unwrap();
                            tmp_stream.is_default =
                                XMLParser::get_value_from_name(&stream.attributes, "isDefault")
                                    .parse::<bool>()
                                    .unwrap();
                            tmp_stream.is_forced =
                                XMLParser::get_value_from_name(&stream.attributes, "isForced")
                                    .parse::<bool>()
                                    .unwrap();
                            tmp_stream.language =
                                XMLParser::get_value_from_name(&stream.attributes, "language");
                            tmp_stream.frame_width =
                                XMLParser::get_value_from_name(&stream.attributes, "width")
                                    .parse::<u16>()
                                    .unwrap();
                            tmp_stream.frame_height =
                                XMLParser::get_value_from_name(&stream.attributes, "height")
                                    .parse::<u16>()
                                    .unwrap();
                            tmp_stream.sample_rate = XMLParser::get_value_from_name(
                                &stream.attributes,
                                "sampleFrequenzy",
                            ).parse::<u32>()
                                .unwrap();
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
                                XMLParser::get_value_from_name(&stream.attributes, "size")
                                    .parse::<u64>()
                                    .unwrap();
                            tmp_thumb.height =
                                XMLParser::get_value_from_name(&stream.attributes, "height")
                                    .parse::<u16>()
                                    .unwrap();
                            tmp_thumb.width =
                                XMLParser::get_value_from_name(&stream.attributes, "width")
                                    .parse::<u16>()
                                    .unwrap();
                            tmp_thumb.item_id =
                                XMLParser::get_value_from_name(&stream.attributes, "itemId")
                                    .parse::<u64>()
                                    .unwrap();
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
                if self.does_exist(&tmp_item.file_path) {
                    // Make sure it was not modified -> do not add if so -> Needs to be parsed anyway so we do not need to check the file again
                    if DatabaseManager::get_last_modified(&tmp_item.file_path) <=
                        tmp_item.last_modified
                    {
                        self.media_item.push(tmp_item);
                    }
                }
            }
        }
    }

    /// Takes the given ID and sets it to be the highest one
    /// if it is greater than the last one.
    ///
    /// This is used to make sure the DatabaseManager always nows
    /// the last ID and is able to offer a new ID that is always
    /// unique over all Elements.
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
    fn get_last_modified(path: &str) -> u64 {
        let metadata = fs::metadata(path);
        metadata
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Checks if a File or Folder does exist. Returns
    /// true if so and false if not.
    fn does_exist(&self, path: &str) -> bool {
        let path = Path::new(path);
        if path.exists() { true } else { false }
    }
}

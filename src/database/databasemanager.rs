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
    pub fn new(db_path: &str, shares: Vec<String>) -> DatabaseManager {
        DatabaseManager {
            path: db_path.to_string(),
            media_item: Vec::new(),
            media_folders: Vec::new(),
            share_folders: Vec::new(),
            media_formats: Vec::new(),
            latest_id: 0,
            parser: MediaParser::new(),
        }
    }

    fn save_database(&self) {
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
            xml_parser.open_tag("folder", &folder.get_name_value_pairs(), true);

            // Go through all Items
            for item in &self.media_item {
                // Write only the Items inside current Foder
                if item.parent_id == folder.id {
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

        let mut db_file = File::open(&self.path).expect("Unable to open Database");
        db_file.write(xml_parser.xml_content.as_bytes()).expect(
            "Unable to write to Database!",
        );
    }

    fn load_database(&mut self) {
        // Open Database File
        let mut db_file = File::open(&self.path).expect("Unable to open Database!");
        let mut contents = String::new();
        db_file.read_to_string(&mut contents).expect(
            "something went wrong reading the file",
        );

        let mut xml_parser: XMLParser = XMLParser::open(&contents);
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
                let tmp_path: String = tmp_item.file_path.clone();
                let path = Path::new(&tmp_path);
                if path.exists() {
                    // Make sure it was not modified -> do not add if so
                    let metadata = fs::metadata(&tmp_path);
                    let modified_date = metadata
                        .unwrap()
                        .modified()
                        .unwrap()
                        .duration_since(time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    if modified_date > tmp_item.last_modified {
                        self.media_item.push(tmp_item);
                    }
                }
            }
        }
    }

    fn set_latest_id(&mut self, id: u64) {
        if id > self.latest_id {
            self.latest_id = id;
        }
    }
}

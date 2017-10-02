use std::cmp::Ordering;
use configuration::ConfigurationHandler;
use database::{DatabaseManager, Folder};
use tools::{XMLParser, NameValuePair};
use media::Item;

/// # ContentDirectory
///
/// This is the Implementation of the UPnP Content Directory
/// Service as required by UPnP. Additionally this is the
/// Connection between the Servers Media Database and the
/// Renderers Request.
/// Every Content requested will be gathered and provided here.
pub struct ContentDirectory<'a, 'b> {
    cfg_handler: &'a ConfigurationHandler,
    db_handler: &'b mut DatabaseManager,
    xml_parser: XMLParser,
    system_update_id: u64,
}

impl<'a, 'b> ContentDirectory<'a, 'b> {
    /// Creates a new Content Directory Structure with the given
    /// Values.
    ///
    /// # Arguments
    ///
    /// * `cfg_handler` - Configuration Handler that provides any Configuration needed here
    /// * `db_handler` - Database Handler that provides Media DB Access
    pub fn new(
        cfg_handler: &'a ConfigurationHandler,
        db_handler: &'b mut DatabaseManager,
    ) -> ContentDirectory<'a, 'b> {
        ContentDirectory {
            cfg_handler: cfg_handler,
            db_handler: db_handler,
            xml_parser: XMLParser::new(),
            system_update_id: 1,
        }
    }

    /// Takes an incoming request and generates the corresponding XML Answeer.
    /// If the request could not be processed the Answer will be an empty String.
    ///
    /// # Arguments
    ///
    /// * `request` - Request received from a Renderer including Header and Content
    pub fn handle_request(&mut self, request: &str) -> String {
        if &request[..36] == "SUBSCRIBE /content/content_directory" {
            return self.do_subscribe();
        } else if request.find("u:GetSearchCapabilities").is_some() {
            return self.get_search_capabilities();
        } else if request.find("u:GetSortCapabilities").is_some() {
            return self.get_sort_capabilities();
        } else if request.find("u:Browse").is_some() {
            if request.find("BrowseMetadata").is_some() {
                return self.browser_direct_child(request);
            } else {
                return self.browse(request);
            }
        } else if request.find("u:Search").is_some() {
            if request.find("SearchMetadata").is_some() {
                return self.search_direct_children();
            } else {
                return self.search();
            }
        } else if request.find("u:GetSystemUpdateID").is_some() {
            return self.get_system_update_id();
        } else {
            return String::new();
        }
    }

    fn sort_folders(folders: &mut Vec<Folder>, criteria: &str) {

        if criteria.is_empty() {
            folders.sort_by(|a, b| a.title.cmp(&b.title));
            return;
        }

        let orders: Vec<&str> = criteria.split(",").collect();

        folders.sort_by(|a, b| {
            orders.iter().fold(Ordering::Equal, |acc, &field| {
                acc.then_with(|| match field {
                    "+dc:title" => a.title.cmp(&b.title),
                    "+dc:date" => a.last_modified.cmp(&b.last_modified),
                    "-dc:title" => a.title.cmp(&b.title).reverse(),
                    "-dc:date" => a.last_modified.cmp(&b.last_modified).reverse(),
                    _ => a.title.cmp(&b.title),
                })
            })
        });
    }

    fn sort_items(items: &mut Vec<Item>, criteria: &str) {

        if criteria.is_empty() {
            items.sort_by(|a, b| a.meta_data.file_name.cmp(&b.meta_data.file_name));
            return;
        }

        let orders: Vec<&str> = criteria.split(",").collect();

        items.sort_by(|a, b| {
            orders.iter().fold(Ordering::Equal, |acc, &field| {
                acc.then_with(|| match field {
                    "+dc:title" => a.meta_data.file_name.cmp(&b.meta_data.file_name),
                    "+upnp:genre" => a.meta_data.genre.cmp(&b.meta_data.genre),
                    "+dc:date" => a.last_modified.cmp(&b.last_modified),
                    "+dc:description" => a.meta_data.description.cmp(&b.meta_data.description),
                    "+upnp:longDescription" => {
                        a.meta_data.long_description.cmp(
                            &b.meta_data.long_description,
                        )
                    }
                    "+upnp:producer" => a.meta_data.producer.cmp(&b.meta_data.producer),
                    "+upnp:rating" => a.meta_data.rating.cmp(&b.meta_data.rating),
                    "+upnp:actor" => a.meta_data.actor.cmp(&b.meta_data.actor),
                    "+upnp:director" => a.meta_data.director.cmp(&b.meta_data.director),
                    "+dc:publisher" => a.meta_data.publisher.cmp(&b.meta_data.publisher),
                    "+upnp:album" => a.meta_data.album.cmp(&b.meta_data.album),
                    "+upnp:originalTrackNumber" => {
                        a.meta_data.track_number.cmp(&b.meta_data.track_number)
                    }
                    "+upnp:playlist" => a.meta_data.playlist.cmp(&b.meta_data.playlist),
                    "+dc:contributor" => a.meta_data.contributor.cmp(&b.meta_data.contributor),
                    "+dc:language" => {
                        if a.meta_data.languages.len() > 0 && b.meta_data.languages.len() > 0 {
                            a.meta_data.languages.get(0).unwrap().cmp(&b.meta_data
                                .languages
                                .get(0)
                                .unwrap())
                        } else {
                            a.meta_data.file_name.cmp(&b.meta_data.file_name)
                        }
                    }
                    "+upnp:artist" => {
                        if a.meta_data.artists.len() > 0 && b.meta_data.artists.len() > 0 {
                            a.meta_data.artists.get(0).unwrap().cmp(&b.meta_data
                                .artists
                                .get(0)
                                .unwrap())
                        } else {
                            a.meta_data.file_name.cmp(&b.meta_data.file_name)
                        }
                    }
                    "+dc:rights" => {
                        if a.meta_data.copyrights.len() > 0 && b.meta_data.copyrights.len() > 0 {
                            a.meta_data.copyrights.get(0).unwrap().cmp(&b.meta_data
                                .copyrights
                                .get(0)
                                .unwrap())
                        } else {
                            a.meta_data.file_name.cmp(&b.meta_data.file_name)
                        }
                    }
                    "-dc:title" => a.meta_data.file_name.cmp(&b.meta_data.file_name).reverse(),
                    "-upnp:genre" => a.meta_data.genre.cmp(&b.meta_data.genre).reverse(),
                    "-dc:date" => a.last_modified.cmp(&b.last_modified).reverse(),
                    "-dc:description" => {
                        a.meta_data
                            .description
                            .cmp(&b.meta_data.description)
                            .reverse()
                    }
                    "-upnp:longDescription" => {
                        a.meta_data
                            .long_description
                            .cmp(&b.meta_data.long_description)
                            .reverse()
                    }
                    "-upnp:producer" => a.meta_data.producer.cmp(&b.meta_data.producer).reverse(),
                    "-upnp:rating" => a.meta_data.rating.cmp(&b.meta_data.rating).reverse(),
                    "-upnp:actor" => a.meta_data.actor.cmp(&b.meta_data.actor).reverse(),
                    "-upnp:director" => a.meta_data.director.cmp(&b.meta_data.director).reverse(),
                    "-dc:publisher" => a.meta_data.publisher.cmp(&b.meta_data.publisher).reverse(),
                    "-upnp:album" => a.meta_data.album.cmp(&b.meta_data.album).reverse(),
                    "-upnp:originalTrackNumber" => {
                        a.meta_data
                            .track_number
                            .cmp(&b.meta_data.track_number)
                            .reverse()
                    }
                    "-upnp:playlist" => a.meta_data.playlist.cmp(&b.meta_data.playlist).reverse(),
                    "-dc:contributor" => {
                        a.meta_data
                            .contributor
                            .cmp(&b.meta_data.contributor)
                            .reverse()
                    }
                    "-dc:language" => {
                        if a.meta_data.languages.len() > 0 && b.meta_data.languages.len() > 0 {
                            a.meta_data
                                .languages
                                .get(0)
                                .unwrap()
                                .cmp(&b.meta_data.languages.get(0).unwrap())
                                .reverse()
                        } else {
                            a.meta_data.file_name.cmp(&b.meta_data.file_name).reverse()
                        }
                    }
                    "-upnp:artist" => {
                        if a.meta_data.artists.len() > 0 && b.meta_data.artists.len() > 0 {
                            a.meta_data
                                .artists
                                .get(0)
                                .unwrap()
                                .cmp(&b.meta_data.artists.get(0).unwrap())
                                .reverse()
                        } else {
                            a.meta_data.file_name.cmp(&b.meta_data.file_name).reverse()
                        }
                    }
                    "-dc:rights" => {
                        if a.meta_data.copyrights.len() > 0 && b.meta_data.copyrights.len() > 0 {
                            a.meta_data
                                .copyrights
                                .get(0)
                                .unwrap()
                                .cmp(&b.meta_data.copyrights.get(0).unwrap())
                                .reverse()
                        } else {
                            a.meta_data.file_name.cmp(&b.meta_data.file_name).reverse()
                        }
                    }

                    _ => a.meta_data.file_name.cmp(&b.meta_data.file_name),
                })
            })
        });
    }


    /// Handles the Renderers Browse Requests and will generate a List of Results.
    /// if something went wrong this function will return an empty String.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming Request from the Renderer
    fn browse(&mut self, request: &str) -> String {
        // Check if there are renderers
        if self.cfg_handler.renderer_configurations.len() == 0 {
            return String::new();
        }

        let empty_vec: Vec<NameValuePair> = Vec::new();
        let mut content: String = String::from(
            "&lt;DIDL-Lite xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\" 
            xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\"&gt;",
        );

        self.xml_parser.start_xml();
        self.xml_parser.open_tag(
            "s:Envelope",
            &vec![
                NameValuePair::new(
                    "xmlns:s",
                    "http://schemas.xmlsoap.org/soap/envelope/"
                ),
                NameValuePair::new(
                    "s:encodingStyle",
                    "http://schemas.xmlsoap.org/soap/encoding/"
                ),
            ],
            true,
        );
        self.xml_parser.open_tag("s:Body", &empty_vec, true);
        self.xml_parser.open_tag(
            "u:BrowseResponse",
            &vec![
                NameValuePair::new(
                    "xmlns:u",
                    "urn:schemas-upnp-org:service:ContentDirectory:1"
                ),
            ],
            true,
        );
        self.xml_parser.open_tag("Result", &empty_vec, true);

        let id: u64 = match self.find_value_from_name(request, "ObjectID")
            .parse::<u64>() {
            Ok(value) => value,
            Err(_) => return String::new(),
        };
        let start_index: usize = match self.find_value_from_name(request, "StartingIndex")
            .parse::<usize>() {
            Ok(value) => value,
            Err(_) => return String::new(),
        };
        let requested_count: u64 = match self.find_value_from_name(request, "RequestedCount")
            .parse::<u64>() {
            Ok(value) => value,
            Err(_) => return String::new(),
        };
        let sort_criteria: String = self.find_value_from_name(request, "SortCriteria");

        let mut act_count: u64 = 0;
        let mut item_index: usize = start_index;

        let mut folders: Vec<Folder> = self.db_handler.get_folder_from_parent(id);
        ContentDirectory::sort_folders(&mut folders, &sort_criteria);

        for index in start_index..folders.len() {
            if act_count < requested_count || requested_count == 0 {
                content.push_str(&folders[index].generate_upnp_xml());
                act_count += 1;
            } else {
                break;
            }
        }

        let mut items: Vec<Item> = self.db_handler.get_items_from_parent(id);
        ContentDirectory::sort_items(&mut items, &sort_criteria);

        if act_count > 0 {
            item_index = 0;
        }

        for index in item_index..items.len() {
            if act_count < requested_count || requested_count == 0 {
                content.push_str(&items[index].generate_upnp_xml(
                    match self.cfg_handler
                        .renderer_configurations
                        .get(0) {
                        Some(value) => value,
                        None => return String::new(),
                    },
                    &self.cfg_handler.server_configuration,
                ));
                act_count += 1;
            } else {
                break;
            }
        }

        content.push_str("&lt;/DIDL-Lite&gt;");
        self.xml_parser.insert_value(&content);

        self.xml_parser.close_tag("Result");

        self.xml_parser.open_tag("NumberReturned", &empty_vec, true);
        self.xml_parser.insert_value(&act_count.to_string());
        self.xml_parser.close_tag("NumberReturned");

        self.xml_parser.open_tag("TotalMatches", &empty_vec, true);
        self.xml_parser.insert_value(
            &(folders.len() + items.len()).to_string(),
        );
        self.xml_parser.close_tag("TotalMatches");

        self.xml_parser.open_tag("UpdateID", &empty_vec, true);

        let mut update_id = 2;
        if act_count == 0 {
            update_id = 1;
        }

        self.xml_parser.insert_value(&update_id.to_string());
        self.xml_parser.close_tag("UpdateID");

        self.xml_parser.close_tag("u:BrowseResponse");
        self.xml_parser.close_tag("s:Body");
        self.xml_parser.close_tag("s:Envelope");

        self.xml_parser.xml_content.clone()
    }

    /// Handles the direct children Browse Request of a Media Renderer.
    /// Returns an empty String if something went wrong.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming Request from a Renderer
    fn browser_direct_child(&mut self, request: &str) -> String {
        let empty_vec: Vec<NameValuePair> = Vec::new();
        let mut content: String = String::from(
            "&lt;DIDL-Lite xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\" 
            xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\"&gt;",
        );

        self.xml_parser.start_xml();
        self.xml_parser.open_tag(
            "s:Envelope",
            &vec![
                NameValuePair::new(
                    "xmlns:s",
                    "http://schemas.xmlsoap.org/soap/envelope/"
                ),
                NameValuePair::new(
                    "s:encodingStyle",
                    "http://schemas.xmlsoap.org/soap/encoding/"
                ),
            ],
            true,
        );
        self.xml_parser.open_tag("s:Body", &empty_vec, true);
        self.xml_parser.open_tag(
            "u:BrowseResponse",
            &vec![
                NameValuePair::new(
                    "xmlns:u",
                    "urn:schemas-upnp-org:service:ContentDirectory:1"
                ),
            ],
            true,
        );
        self.xml_parser.open_tag("Result", &empty_vec, true);

        let id: u64 = match self.find_value_from_name(request, "ObjectID")
            .parse::<u64>() {
            Ok(value) => value,
            Err(_) => return String::new(),
        };

        let mut result_nb = 0;

        match self.db_handler.get_folder_direct(id) {
            Ok(folder) => {
                content.push_str(&folder.generate_upnp_xml());
                result_nb = 1;
            }
            Err(_) => {
                match self.db_handler.get_item_direct(id) {
                    Ok(item) => {
                        content.push_str(&item.generate_upnp_xml(
                            match self.cfg_handler.renderer_configurations.get(0) {
                                Some(value) => value,
                                None => return String::new(),
                            },
                            &self.cfg_handler.server_configuration,
                        ));
                        result_nb = 1;
                    }
                    Err(_) => {}
                }
            }
        }

        content.push_str("&lt;/DIDL-Lite&gt;");
        self.xml_parser.insert_value(&content);

        self.xml_parser.close_tag("Result");

        self.xml_parser.open_tag("NumberReturned", &empty_vec, true);
        self.xml_parser.insert_value(&result_nb.to_string());
        self.xml_parser.close_tag("NumberReturned");

        self.xml_parser.open_tag("TotalMatches", &empty_vec, true);
        self.xml_parser.insert_value(&result_nb.to_string());
        self.xml_parser.close_tag("TotalMatches");

        self.xml_parser.open_tag("UpdateID", &empty_vec, true);

        let mut update_id = 2;
        if result_nb == 0 {
            update_id = 1;
        }

        self.xml_parser.insert_value(&update_id.to_string());
        self.xml_parser.close_tag("UpdateID");

        self.xml_parser.close_tag("u:BrowseResponse");
        self.xml_parser.close_tag("s:Body");
        self.xml_parser.close_tag("s:Envelope");

        self.xml_parser.xml_content.clone()
    }

    /// Perfomrs the Subscribe Action
    ///
    /// # TO-DO
    /// - Actually subscribe
    fn do_subscribe(&self) -> String {
        format!(
            "<e:propertyset xmlns:e=\"urn:schemas-upnp-org:event-1-0\" xmlns:s=\"urn:schemas-upnp-org:service:ContentDirectory:1\">
            <e:property>
            <TransferIDs>
            </TransferIDs>
            </e:property>
            <e:property>
            <ContainerUpdateIDs>
            </ContainerUpdateIDs>
            </e:property>
            <e:property>
            <SystemUpdateID>{}</SystemUpdateID>
            </e:property>
            </e:propertyset>",
            self.system_update_id
        )
    }

    /// Returns the Search Capabilities of the Server
    fn get_search_capabilities(&self) -> String {
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>
         <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/ s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"\">
	         <s:Body>
		         <u:GetSearchCapabilitiesResponse xmlns:u=\"urn:schemas-upnp-org:service:ContentDirectory:1\">
			         <SearchCaps>
				         *
			         </SearchCaps>
		         </u:GetSearchCapabilitiesResponse>
	         </s:Body>
         </s:Envelope>".to_string()
    }

    /// Returns the Sort Capabilities of the Server
    fn get_sort_capabilities(&self) -> String {
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>
         <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/ s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"\">
	         <s:Body>
		         <u:GetSortCapabilitiesResponse xmlns:u=\"urn:schemas-upnp-org:service:ContentDirectory:1\">
			         <SortCaps>
				         *
			         </SortCaps>
		         </u:GetSortCapabilitiesResponse>
	         </s:Body>
         </s:Envelope>".to_string()
    }

    /// Returns the current Update Id of the Content
    ///
    /// # TO-DO
    /// - Actually keep track of this
    fn get_system_update_id(&self) -> String {
        format!(
            "<?xml version=\"1.0\" encoding=\"utf-8\"?>
         <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/ s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"\">
	         <s:Body>
		         <u:GetSystemUpdateIDResponse xmlns:u=\"urn:schemas-upnp-org:service:ContentDirectory:1\">
			         <Id>{}</Id>
		         </u:GetSystemUpdateIDResponse>
	         </s:Body>
         </s:Envelope>",
            self.system_update_id
        )
    }

    /// Searches for the given Value in an Request and returns its XML Content.
    /// Returns an empty String if nothing was found.
    ///
    /// # Arguments
    ///
    /// * `request` - The incoming Request from a Renderer
    fn find_value_from_name(&self, request: &str, name: &str) -> String {
        let start_search = format!("<{}>", name);
        let end_search = format!("</{}>", name);

        match request.find(&start_search) {
            Some(start_position) => {
                match request.find(&end_search) {
                    Some(end_position) => {
                        if start_position + start_search.len() >= request.len() &&
                            end_position >= request.len()
                        {
                            return String::new();
                        }

                        return request[start_position + start_search.len()..end_position]
                            .to_string();
                    }
                    None => {
                        return String::new();
                    }
                }
            }
            None => {
                return String::new();
            }
        }
    }

    /// Performs the Search Request of a Renderer and returns the Resulst as XML.
    ///
    /// # TO-DO
    /// -Actually implement this
    fn search(&self) -> String {
        String::new()
    }

    /// Performs the direct children Search Requst of a Renderer and returns the Results as XML.
    ///
    /// # TO-DO
    /// -Actually implement this
    fn search_direct_children(&self) -> String {
        String::new()
    }
}

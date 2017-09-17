use configuration::ConfigurationHandler;
use database::DatabaseManager;
use tools::XMLParser;
use tools::NameValuePair;
use database::Folder;
use media::Item;

pub struct ContentDirectory<'a, 'b> {
    cfg_handler: &'a ConfigurationHandler,
    db_handler: &'b DatabaseManager,
    xml_parser: XMLParser,
    system_update_id: u64,
}

impl<'a, 'b> ContentDirectory<'a, 'b> {
    pub fn new(
        cfg_handler: &'a ConfigurationHandler,
        db_handler: &'b DatabaseManager,
    ) -> ContentDirectory<'a, 'b> {
        ContentDirectory {
            cfg_handler: cfg_handler,
            db_handler: db_handler,
            xml_parser: XMLParser::new(),
            system_update_id: 1,
        }
    }

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

    fn browse(&mut self, request: &str) -> String {
        let empty_vec: Vec<NameValuePair> = Vec::new();
        let mut content: String = String::from(
            "&lt;DIDL-Lite xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\"&gt;",
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

        let id: u64 = self.find_value_from_name(request, "ObjectID")
            .parse::<u64>()
            .unwrap();
        let start_index: usize = self.find_value_from_name(request, "StartingIndex")
            .parse::<usize>()
            .unwrap();
        let requested_count: u64 = self.find_value_from_name(request, "RequestedCount")
            .parse::<u64>()
            .unwrap();

        let mut act_count: u64 = 0;
        let mut item_index: usize = 0;

        let mut folders: Vec<Folder> = self.db_handler.get_folder_from_parent(id);
        folders.sort_by(|a, b| a.title.cmp(&b.title));

        for index in start_index..folders.len() {
            if act_count < requested_count || requested_count == 0 {
                content.push_str(&folders[index].generate_upnp_xml());
                act_count += 1;
            } else {
                break;
            }
        }

        let mut items: Vec<Item> = self.db_handler.get_items_from_parent(id);
        items.sort_by(|a, b| a.meta_data.file_name.cmp(&b.meta_data.file_name));

        if act_count > 0 {
            item_index = 0;
        } else {
            item_index = start_index;
        }

        for index in item_index..items.len() {
            if act_count < requested_count || requested_count == 0 {
                content.push_str(&items[index].generate_upnp_xml(
                    &self.cfg_handler.renderer_configurations[0],
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

        let mut update_id = 0;
        if act_count == 0 {
            update_id = 1;
        } else {
            update_id = 2;
        }

        self.xml_parser.insert_value(&update_id.to_string());
        self.xml_parser.close_tag("UpdateID");

        self.xml_parser.close_tag("u:BrowseResponse");
        self.xml_parser.close_tag("s:Body");
        self.xml_parser.close_tag("s:Envelope");

        self.xml_parser.xml_content.clone()
    }

    fn browser_direct_child(&mut self, request: &str) -> String {
        let empty_vec: Vec<NameValuePair> = Vec::new();
        let mut content: String = String::from(
            "&lt;DIDL-Lite xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\"&gt;",
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

        let id: u64 = self.find_value_from_name(request, "ObjectID")
            .parse::<u64>()
            .unwrap();

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
                            &self.cfg_handler.renderer_configurations[0],
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

        let mut update_id = 0;
        if result_nb == 0 {
            update_id = 1;
        } else {
            update_id = 2;
        }

        self.xml_parser.insert_value(&update_id.to_string());
        self.xml_parser.close_tag("UpdateID");

        self.xml_parser.close_tag("u:BrowseResponse");
        self.xml_parser.close_tag("s:Body");
        self.xml_parser.close_tag("s:Envelope");

        self.xml_parser.xml_content.clone()
    }

    fn do_subscribe(&self) -> String {
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
            <SystemUpdateID>".to_string() + &self.system_update_id.to_string() + &"</SystemUpdateID>
            </e:property>
            </e:propertyset>".to_string()
    }

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

    fn get_system_update_id(&self) -> String {
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>
         <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/ s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"\">
	         <s:Body>
		         <u:GetSystemUpdateIDResponse xmlns:u=\"urn:schemas-upnp-org:service:ContentDirectory:1\">
			         <Id>".to_string() + &self.system_update_id.to_string() + &"
			         </Id>
		         </u:GetSystemUpdateIDResponse>
	         </s:Body>
         </s:Envelope>".to_string()
    }

    fn find_value_from_name(&self, request: &str, name: &str) -> String {
        let mut start_search = String::from("<");
        start_search.push_str(name);
        start_search.push_str(">");

        let mut end_search = String::from("</");
        end_search.push_str(name);
        end_search.push_str(">");

        match request.find(&start_search) {
            Some(start_position) => {
                match request.find(&end_search) {
                    Some(end_position) => {
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

    fn search(&self) -> String {
        String::new()
    }

    fn search_direct_children(&self) -> String {
        String::new()
    }
}

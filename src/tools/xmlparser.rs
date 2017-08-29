use super::namevaluepair::NameValuePair;

/// # XMLEntry
///
/// This structure holds all Data required for a XML Tag
/// including the Tag (name) itself, a Value if no Sub-Tags
/// are present, a list of Attributes as Name-Value-Pairs
/// and a list of Sub-Tags each of the same XMLEntry struct type.
pub struct XMLEntry {
    pub tag: String,
    pub value: String,
    pub attributes: Vec<NameValuePair>,
    pub sub_tags: Vec<XMLEntry>,
}

impl XMLEntry {
    /// Creates a new and empty XMLEntry structure
    pub fn new() -> XMLEntry {
        XMLEntry {
            tag: String::new(),
            value: String::new(),
            attributes: Vec::new(),
            sub_tags: Vec::new(),
        }
    }

    /// Copys this XMLEntry and creates a new one.
    /// Both Entries can than be used independantly.
    ///
    /// # Example
    ///
    /// ```
    /// let other_xml: XMLEntry = XMLEntry::new();
    /// let new_xml: XMLEntry = other_xml.copy();
    /// ```
    pub fn copy(&self) -> XMLEntry {
        let mut entry: XMLEntry = XMLEntry::new();

        entry.tag = self.tag.clone();
        entry.value = self.value.clone();

        for attr in &self.attributes {
            entry.attributes.push(attr.copy());
        }

        for sub in &self.sub_tags {
            entry.sub_tags.push(sub.copy());
        }

        return entry;
    }
}

/// # XMLParser
///
/// This structure implements all functions required to work with
/// XML Data.
/// You can parse a XML formated String into a list of XMLEntrys or
/// create a new XML String.
///
/// ## Parsing XML Strings
/// ### 1. Method
/// Create a XMLParser and call the parse() function in order to get a
/// list of XMLEntrys directly. The Parser will not store any XMLEntrys
/// itself!
///
/// ```
/// use tools::xmlparser::XMLParser;
/// use tools::xmlparser::XMLEntry;
/// let parser: XMLParser = XMLParser::new();
/// let tag_list:Vec<XMLEntry> = parser.parse("<?xml...");
/// ```
///
/// ### 2. Method (Recommended)
/// Parse directly on creation and store the XMLEntrys. Use the open()
/// function instead of new() and provide the XML Data.The Tags are than
/// available through the Parsers xml_entries attribute.
///
/// ```
/// use tools::xmlparser::XMLParser;
/// let parser: XMLParser = XMLParser::open("<?xml...");
/// for tag in parser.xml_entries {}
/// ```
///
/// ## Creating XML Strings
/// To create a new XML formatted String first call open_xml(), add tags with attributes,
/// values and Sub-Tags. Than use the Parsers xml_content attribute to get the XML
/// Data.
///
/// ```
/// use tools::namevaluepair::NameValuePair;
/// use tools::xmlparser::XMLParser;
///
/// let mut attributes: Vec<NameValuePair> = Vec::new();
/// attributes.push(NameValuePair::new("attr_name", "attr_value"));
///
/// let mut parser: XMLParser = XMLParser::new();
/// parser.start_xml();
/// parser.open_tag("main_tag", attributes, true);
/// parser.open_tag("sub_tag", attributes, false);
/// parser.open_tag("sub_tag_2", attributes, true);
/// parser.insert_value("value_sub_tag_2");
/// parser.close_tag("subt_tag_2");
/// parser.close_tag("main_tag");
///
/// let xml_data: String = parser.xml_content;
/// ```
///
/// This will produce the following XML Output:
///
/// ```
/// <?xml version="1.0" encoding="UTF-8"?>
/// <main_tag attr_name="attr_value">
/// 	<sub_tag attr_name="attr_value"/>
/// 	<sub_tag_2 attr_name="attr_value">
///			value_sub_tag_2
/// 	</sub_tag_2>
/// </main_tag>
pub struct XMLParser {
    pub xml_content: String,
    pub xml_entries: Vec<XMLEntry>,
    tab_counter: u32,
}

impl XMLParser {
    /// Initialises a new and empty XMLParser
    pub fn new() -> XMLParser {
        XMLParser {
            xml_content: String::new(),
            xml_entries: Vec::new(),
            tab_counter: 0,
        }
    }

    /// Creates a new XMLParser and directly parses the given
    /// XML String. The results are available through the
    /// Parsers xml_entries attribute.
    ///
    /// # Arguments
    ///
    /// * `content` - A String containing the XML Data
    pub fn open(content: &str) -> XMLParser {
        let mut parser = XMLParser::new();
        parser.xml_content = content.to_string();
        parser.xml_entries = parser.parse(content);
        parser.tab_counter = 0;

        return parser;
    }

    /// Finds the given needle inside a String starting from the given position and writes
    /// the position of the first occurence back into the target. If the needle was
    /// found this function returns true and false if not.
    ///
    /// # Arguments
    ///
    /// * `content` - The String to search in
    /// * `find` - The String to find in "content"
    /// * `position` - The character index to start searching from
    /// * `target` - The target that stores the position of the search term
    fn find_after(&self, content: &str, find: &str, position: usize, target: &mut usize) -> bool {
        match content[position..].find(find) {
            Some(length) => {
                *target = position + length;
                true
            }
            None => false,
        }
    }

    /// Takes a XML Formated String and parses it into a list of XMLEntry structures. Important:
    /// This function does *not* write the results back into the Parser. You need to do it manually
    /// or initialise the Parser with the ::open() function.
    ///
    /// # Arguments
    ///
    /// * `content` - The XML Content to parse
    pub fn parse(&self, content: &str) -> Vec<XMLEntry> {
        let mut start_position: usize = 0;
        let mut end_position: usize = content.len();
        let mut tag_end_position: usize = 0;
        let mut result_list: Vec<XMLEntry> = Vec::new();

        // If there is no Content -> abort
        if content.len() == 0 {
            return result_list;
        }

        // Find the beginning of a new Tag
        while self.find_after(content, "<", start_position, &mut start_position) {

            let mut result: XMLEntry = XMLEntry::new();

            // If this is the End Tag return the Result
            if &content[(start_position + 1)..(start_position + 2)] == "/" {
                return result_list;
            }

            // If this is the <?xml start skip it
            if &content[(start_position + 1)..(start_position + 2)] == "?" {
                if self.find_after(content, "?>", start_position, &mut start_position) == true {
                    start_position += 2;
                    continue;
                }
            }

            // Find the closing Point and break if there is none
            if self.find_after(content, ">", start_position, &mut end_position) == false {
                break;
            }

            // Split on whitespaces to get tag name and attributes
            let mut sub_content: &str = &content[start_position..end_position];

            // First item should be the Tag itself
            let vec: Vec<&str> = sub_content.split(" ").collect();

            match vec.get(0) {
                Some(text) => result.tag = text[1..].to_string(),
                None => (),
            }

            // Parse the Attributes
            result.attributes = self.get_attributes(&mut sub_content);

            // Find the end of this Tag
            let mut end_tag = String::from("</");
            end_tag.push_str(&result.tag);

            if self.find_after(content, &end_tag, end_position, &mut tag_end_position) == true {
                tag_end_position -= 2;

                // Get the Sub Tags
                result.sub_tags = self.parse(&content[(end_position + 1)..(tag_end_position + 1)]);

                if result.sub_tags.len() == 0 {
                    result.value = content[end_position..tag_end_position].trim().to_string();
                }

                start_position = tag_end_position + 3 + result.tag.len();

            } else {
                if self.find_after(content, "/>", start_position, &mut tag_end_position) == true {
                    start_position = tag_end_position + 2;
                }

            }

            result_list.push(result);
        }

        return result_list;
    }

    /// This function takes a single XML Tag and extracts the Attributes (if existing) as
    /// a list of Name-Value-Pairs. If there are no Attributes inside the Tag the list will
    /// be empty.
    ///
    /// * `tag` - The Tag to get the attributes from. -- "<name att1="val1" ...(/)>
    fn get_attributes(&self, tag: &mut &str) -> Vec<NameValuePair> {

        let mut result: Vec<NameValuePair> = Vec::new();
        let mut tmp_string: &str = tag;

        // Remove the leading "<"
        match tmp_string.find("<") {
            Some(length) => tmp_string = &tmp_string[(length + 1)..],
            None => (),
        }

        // Remove the tailing ">" or "/>"
        match tmp_string.find(">") {
            Some(length) => {
                tmp_string = &tmp_string[0..length];
            }
            None => (),
        }

        if &tmp_string[(tmp_string.len() - 1)..] == "/" {
            tmp_string = &tmp_string[..tmp_string.len() - 1];
        }

        // Remove the Tags Name
        match tmp_string.find(" ") {
            Some(length) => {
                tmp_string = &tmp_string[length + 1..];
            }
            None => (),
        }

        // Split the Attributes
        let part_list: Vec<&str> = tmp_string.split("=\"").collect();

        // Get the Attributes
        for tmp_part in 0..part_list.len() {
            let mut part: String = String::new();
            let mut name: &str = "";
            let mut value: &str = "";

            match part_list.get(tmp_part) {
                Some(text) => part = text.to_string(),
                None => (),
            }

            let name_list: Vec<&str> = part.split(" ").collect();

            match name_list.last() {
                Some(text) => name = text,
                None => (),
            }

            match part_list.get(tmp_part + 1) {
                Some(text) => value = text,
                None => (),
            }

            let mut sub_position: usize = 0;

            if self.find_after(&value, "\"", 0, &mut sub_position) == true && name.len() > 0 {

                value = &value[..sub_position];

                let nm_pair: NameValuePair = NameValuePair::new(name.trim(), value.trim());

                result.push(nm_pair);
            }
        }

        return result;
    }

    /// Starts a new XML Content String. Use this before adding any Tags. This will overwrite
    /// any existing XML Data!
    pub fn start_xml(&mut self) {
        self.xml_content = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".to_string();
    }

    /// This function inserts some Tabs into the actual content.
    /// Even if this was not required, it makes reading the XML Content more
    /// user-friendly.
    fn insert_tab(&mut self) {
        for _ in 0..self.tab_counter {
            self.xml_content += "\t";
        }
    }

    /// Insert the Value to a XML Tag. The Tag needs to be opened before, closed after and shall not
    /// contain any Sub-Tags.
    ///
    /// # Arguments
    ///
    /// * `value` - The Value that should be added to a Tag -- <>value</>
    pub fn insert_value(&mut self, value: &str) {
        self.tab_counter += 1;
        self.insert_tab();
        self.xml_content += value;
        self.xml_content += "\n";
    }

    /// Open a new XML Tag with the given Attributes. If there is no Content than the Tag will be closed
    /// with /> at the end. If there is Content inside this Tag, you need to close it with close_tag().
    ///
    /// # Arguments
    ///
    /// * `name` - The tag name to use. -- <NAME ...>
    /// * `attributes` - List of NameValuePairs to be set as Tag Attributes. -- <.. attr1="val1" attr2="val2" ..>
    /// * `has_content` - Set to true if the Tag will contain any other sub-content. Set to false if not. -- true - <>value</> | false - </>
    pub fn open_tag(&mut self, name: &str, attributes: &Vec<NameValuePair>, has_content: bool) {
        self.tab_counter += 1;
        self.insert_tab();
        self.xml_content += "<";
        self.xml_content.push_str(name);

        if attributes.len() > 0 {
            self.xml_content += " ";
        }

        for attr in attributes {
            self.xml_content += &attr.name;
            self.xml_content += "=\"";
            self.xml_content += &attr.value;
            self.xml_content += "\" ";
        }

        if has_content {
            self.xml_content += ">\n";
        } else {
            self.xml_content += "/>\n";
            self.tab_counter -= 1;
        }
    }

    /// Closes a previously opened tag.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the Tag to close. -- </name>
    pub fn close_tag(&mut self, name: &str) {
        self.xml_content += "</";
        self.xml_content += name;
        self.xml_content += ">\n";

        if self.tab_counter > 0 {
            self.tab_counter -= 1;
        }
    }

    /// Takes a list of XMLEntrys and extracts the one with the given Name.
    /// If there are more Tags with that name, the first one will be returned.
    ///
    /// # Arguments
    ///
    /// * `tag_list` - List of XMLEntrys to search in
    /// * `name` - Name of the Tag to search for
    pub fn find_tag(tag_list: &Vec<XMLEntry>, name: &str) -> XMLEntry {
        let mut entry: XMLEntry = XMLEntry::new();

        for tag in tag_list {
            if tag.tag == name {
                return tag.copy();
            } else {
                entry = XMLParser::find_tag(&tag.sub_tags, name);

                if entry.tag.len() > 0 {
                    return entry;
                }
            }
        }

        return entry;
    }

    /// This function takes a list of Name-Value Pairs and returns the
    /// Value of the Pair where the name matches the given one.
    pub fn get_value_from_name(attr_list: &Vec<NameValuePair>, name: &str) -> String {
        for attr in attr_list {
            if attr.name == name {
                return attr.value.clone();
            }
        }

        "".to_string()
    }
}

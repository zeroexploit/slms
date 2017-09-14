use tools::NameValuePair;

/// # Folder
///
/// This Structure holds all Information required for
/// a Folder and its database representation.
pub struct Folder {
    pub id: u64,
    pub parent_id: u64,
    pub title: String,
    pub path: String,
    pub element_count: u32,
    pub last_modified: u64,
}


impl Folder {
    /// Create an new and empty Folder Structure
    pub fn new() -> Folder {
        Folder {
            id: 0,
            parent_id: 0,
            title: String::new(),
            path: String::new(),
            element_count: 0,
            last_modified: 0,
        }
    }

    /// Get a List of Name-Value Pairs representing this Structures
    /// Attributes. This is used to store the Values inside the
    /// Database.
    pub fn get_name_value_pairs(&self) -> Vec<NameValuePair> {
        let np_list: Vec<NameValuePair> =
            vec![
                NameValuePair::new("id", &self.id.to_string()),
                NameValuePair::new("parentId", &self.parent_id.to_string()),
                NameValuePair::new("title", &self.title),
                NameValuePair::new("path", &self.path),
                NameValuePair::new("count", &self.element_count.to_string()),
                NameValuePair::new("lastModified", &self.last_modified.to_string()),
            ];

        np_list
    }

    /// This will create a clone of the Folder Structure
    /// that can be handled indepentandly.
    pub fn clone(&self) -> Folder {
        Folder {
            id: self.id,
            parent_id: self.parent_id,
            title: self.title.clone(),
            path: self.path.clone(),
            element_count: self.element_count,
            last_modified: self.last_modified,
        }
    }

    pub fn generate_upnp_xml(&self) -> String {
        String::new()
    }
}

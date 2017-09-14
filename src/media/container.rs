use tools;

/// # Container
///
/// Media Container Format that keeps track of all Values
/// the Media Format (like .mkv) contains and are required to
/// serve the correct Formats to the Media Renderers.
///
/// # To-Do
/// This Structure needs to be edited once used and the documentation
/// needs to be updated and proberly written.
#[derive(Clone)]
pub struct Container {
    pub id: u64,
    pub name: String,
    pub file_extensions: Vec<String>,
    pub mime_types: Vec<String>,
}

impl Container {
    /// Create a new Container
    pub fn new() -> Container {
        Container {
            id: 0,
            name: String::from(""),
            file_extensions: Vec::new(),
            mime_types: Vec::new(),
        }
    }

    /// Add a File Extension to the List of Extension this
    /// Container uses
    pub fn add_file_extension(&mut self, extension: &str) {
        self.file_extensions.push(extension.to_string());
    }

    /// Add a Mime Type to the List of Mime Types that might
    /// be used to provide this Container to Media Renderers
    pub fn add_mime_type(&mut self, mime: &str) {
        self.mime_types.push(mime.to_string());
    }

    /// Check if a given File Extension is part of this Container
    /// Format.
    pub fn has_file_extension(&self, extension: &str) -> bool {
        for ex in &self.file_extensions {
            if extension.to_string() == *ex {
                return true;
            }
        }

        false
    }

    /// Check if the given Mime Type is part of this Container
    /// Format.
    pub fn has_mime_type(&self, mime: &str) -> bool {
        for mime_type in &self.mime_types {
            if mime.to_string() == *mime_type {
                return true;
            }
        }

        false
    }

    /// Get a List of Name Value Pairs with the Attributes of this
    /// Media Container Format.
    pub fn get_name_value_pairs(&self) -> Vec<tools::NameValuePair> {
        let pair_vec: Vec<tools::NameValuePair> =
            vec![
                tools::NameValuePair::new("id", &self.id.to_string()),
                tools::NameValuePair::new("name", &self.name),
            ];

        pair_vec
    }
}

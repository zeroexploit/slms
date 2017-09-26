use tools;
use std::path::Path;

/// # Thumbnail
///
/// This structure holds all Information about a Thumbnail
/// and provides a function to create a Thumbnail.
/// This structure is part of a Media Item and should not be
/// used alone.
///
/// # To-Do
/// Add the creation Function once FFMPEG can be directly used.
#[derive(Clone)]
pub struct Thumbnail {
    pub item_id: u64,
    pub file_path: String,
    pub file_size: u64,
    pub mime_type: String,
    pub width: u16,
    pub height: u16,
}

impl Thumbnail {
    /// Create a new and Eepty Thumbnail
    pub fn new() -> Thumbnail {
        Thumbnail {
            item_id: 0,
            file_path: String::new(),
            file_size: 0,
            mime_type: String::new(),
            width: 0,
            height: 0,
        }
    }

    /// Get a List of NameValue Pairs representing the Structures Attributes
    pub fn get_name_value_pairs(&self) -> Vec<tools::NameValuePair> {
        let pair_vec: Vec<tools::NameValuePair> =
            vec![
                tools::NameValuePair::new("itemId", &self.item_id.to_string()),
                tools::NameValuePair::new("path", &self.file_path),
                tools::NameValuePair::new("mimeType", &self.mime_type),
                tools::NameValuePair::new("size", &self.file_size.to_string()),
                tools::NameValuePair::new("width", &self.width.to_string()),
                tools::NameValuePair::new("height", &self.height.to_string()),
            ];

        pair_vec
    }

    /// Check if a Thumbnail is available
    pub fn is_available(&self) -> bool {
        if self.file_path.len() != 0 {
            let path = Path::new(&self.file_path);

            if path.exists() {
                return true;
            }
        }

        false
    }
}

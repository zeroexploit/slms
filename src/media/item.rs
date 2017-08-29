use super::container::Container;
use super::stream::Stream;
use super::thumbnail::Thumbnail;
use tools::NameValuePair;

/// # MediaType
///
/// This enumartion is used to set what kind of
/// Media a File is. E.g. a Movie, Music, etc.
pub enum MediaType {
    UNKNOWN,
    AUDIO,
    PICTURE,
    VIDEO,
}

impl MediaType {
    /// Convert the enumartion into a printable String.
    /// Used in order to store the Information inside
    /// the Media Database.
    pub fn to_string(&self) -> String {
        match *self {
            MediaType::AUDIO => "1".to_string(),
            MediaType::PICTURE => "2".to_string(),
            MediaType::VIDEO => "3".to_string(),
            _ => "0".to_string(),
        }
    }

    /// Convert a printable String back to the enumeration.
    /// Used to load the Information from the
    /// Media Database.
    pub fn from_string(content: &str) -> MediaType {
        match content {
            "1" => MediaType::AUDIO,
            "2" => MediaType::PICTURE,
            "3" => MediaType::VIDEO,
            _ => MediaType::UNKNOWN,
        }
    }
}

/// # MetaData
/// This structure holds all Meta Information that
/// are supported by UPnP and can be extracted from
/// a Media File.
pub struct MetaData {
    pub title: String,
    pub genre: String,
    pub description: String,
    pub long_description: String,
    pub producer: String,
    pub rating: String,
    pub actor: String,
    pub director: String,
    pub publisher: String,
    pub languages: Vec<String>,
    pub artists: Vec<String>,
    pub album: String,
    pub track_number: String,
    pub playlist: String,
    pub contributor: String,
    pub date: String,
    pub copyrights: Vec<String>,
    pub composer: String,
}

impl MetaData {
    /// Create a new MetaData Structure
    pub fn new() -> MetaData {
        MetaData {
            title: String::new(),
            genre: String::new(),
            description: String::new(),
            long_description: String::new(),
            producer: String::new(),
            rating: String::new(),
            actor: String::new(),
            director: String::new(),
            publisher: String::new(),
            languages: Vec::new(),
            artists: Vec::new(),
            album: String::new(),
            track_number: String::new(),
            playlist: String::new(),
            contributor: String::new(),
            date: String::new(),
            copyrights: Vec::new(),
            composer: String::new(),
        }
    }

    /// Get a List of NameValuPairs containing the Media Items
    /// Attributes. Used to store the Information inside the
    /// Media Database
    pub fn get_name_value_pairs(&self) -> Vec<NameValuePair> {
        let mut pair_vec: Vec<NameValuePair> = Vec::new();

        if self.title.len() > 0 {
            pair_vec.push(NameValuePair::new("title", &self.title));
        }

        if self.genre.len() > 0 {
            pair_vec.push(NameValuePair::new("genre", &self.genre));
        }

        if self.description.len() > 0 {
            pair_vec.push(NameValuePair::new("description", &self.description));
        }

        if self.long_description.len() > 0 {
            pair_vec.push(NameValuePair::new(
                "longDescription",
                &self.long_description,
            ));
        }

        if self.producer.len() > 0 {
            pair_vec.push(NameValuePair::new("producer", &self.producer));
        }

        if self.rating.len() > 0 {
            pair_vec.push(NameValuePair::new("rating", &self.rating));
        }

        if self.actor.len() > 0 {
            pair_vec.push(NameValuePair::new("actor", &self.actor));
        }

        if self.director.len() > 0 {
            pair_vec.push(NameValuePair::new("director", &self.director));
        }

        if self.publisher.len() > 0 {
            pair_vec.push(NameValuePair::new("publisher", &self.publisher));
        }

        if self.album.len() > 0 {
            pair_vec.push(NameValuePair::new("album", &self.album));
        }

        if self.track_number.len() > 0 {
            pair_vec.push(NameValuePair::new("trackNumber", &self.track_number));
        }

        if self.playlist.len() > 0 {
            pair_vec.push(NameValuePair::new("playlist", &self.playlist));
        }

        if self.contributor.len() > 0 {
            pair_vec.push(NameValuePair::new("contributor", &self.contributor));
        }

        if self.date.len() > 0 {
            pair_vec.push(NameValuePair::new("date", &self.date));
        }

        if self.composer.len() > 0 {
            pair_vec.push(NameValuePair::new("composer", &self.composer));
        }

        for language in &self.languages {
            pair_vec.push(NameValuePair::new("language", language));
        }

        for artist in &self.artists {
            pair_vec.push(NameValuePair::new("artist", artist));
        }

        for copyright in &self.copyrights {
            pair_vec.push(NameValuePair::new("copyright", copyright));
        }

        pair_vec
    }
}

/// # Item
///
/// This Structure holds all Information, Tracks and Meta Data
/// of a Media File. This is the Place where these kind of Data
/// should be stored.
/// Some parts of these Data is aquired not from the File but the
/// Database. Therefore, all Access should happen through the
/// Database Manager only!
pub struct Item {
    pub id: u64,
    pub parent_id: u64,
    pub last_modified: u64,
    pub file_path: String,
    pub meta_data: MetaData,
    pub media_type: MediaType,
    pub duration: String,
    pub file_size: u64,
    pub media_tracks: Vec<Stream>,
    pub thumbnail: Thumbnail,
    pub format_container: Container,
}

impl Item {
    /// Create a new Item Structure
    pub fn new() -> Item {
        Item {
            id: 0,
            parent_id: 0,
            last_modified: 0,
            file_path: String::new(),
            meta_data: MetaData::new(),
            media_type: MediaType::UNKNOWN,
            duration: String::new(),
            file_size: 0,
            media_tracks: Vec::new(),
            thumbnail: Thumbnail::new(),
            format_container: Container::new(),
        }
    }

    /// Insert new Meta Data.
    /// This is used to parse back the Databases Entries, but not for
    /// reading the Media File itself.
    pub fn insert_meta_data(&mut self, name: &str, value: &str) {
        match name {
            "title" => self.meta_data.title = value.to_string(),
            "genre" => self.meta_data.genre = value.to_string(),
            "description" => self.meta_data.description = value.to_string(),
            "longDescription" => self.meta_data.long_description = value.to_string(),
            "producer" => self.meta_data.producer = value.to_string(),
            "rating" => self.meta_data.rating = value.to_string(),
            "actor" => self.meta_data.actor = value.to_string(),
            "director" => self.meta_data.director = value.to_string(),
            "publisher" => self.meta_data.publisher = value.to_string(),
            "album" => self.meta_data.album = value.to_string(),
            "trackNumber" => self.meta_data.track_number = value.to_string(),
            "playlist" => self.meta_data.playlist = value.to_string(),
            "contributor" => self.meta_data.contributor = value.to_string(),
            "date" => self.meta_data.date = value.to_string(),
            "composer" => self.meta_data.composer = value.to_string(),
            "language" => self.meta_data.languages.push(value.to_string()),
            "artist" => self.meta_data.artists.push(value.to_string()),
            "copyright" => self.meta_data.copyrights.push(value.to_string()),
            _ => (),	
        }
    }

    /// Get the List of NameValuePairs representing this Structures Attributes.
    pub fn get_name_value_pairs(&self) -> Vec<NameValuePair> {
        let pair_vec: Vec<NameValuePair> =
            vec![
                NameValuePair::new("id", &self.id.to_string()),
                NameValuePair::new("parentId", &self.parent_id.to_string()),
                NameValuePair::new("lastModified", &self.last_modified.to_string()),
                NameValuePair::new("path", &self.file_path),
                NameValuePair::new("type", &self.media_type.to_string()),
                NameValuePair::new("duration", &self.duration.to_string()),
                NameValuePair::new("size", &self.file_size.to_string()),
                NameValuePair::new("containerId", &self.format_container.id.to_string()),
            ];

        pair_vec
    }
}

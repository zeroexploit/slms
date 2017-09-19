use super::container::Container;
use super::stream::Stream;
use super::thumbnail::Thumbnail;
use tools::NameValuePair;
use configuration::{RendererConfiguration, ServerConfiguration};

/// # MediaType
///
/// This enumartion is used to set what kind of
/// Media a File is. E.g. a Movie, Music, etc.
#[derive(Clone)]
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
    ///
    /// # Arguments
    ///
    /// * `content` - String representation of the Enum Value
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
///
/// This structure holds all Meta Information that
/// are supported by UPnP and can be extracted from
/// a Media File.
#[derive(Clone)]
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
    pub file_name: String,
    pub file_extension: String,
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
            file_name: String::new(),
            file_extension: String::new(),
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

        if self.file_name.len() > 0 {
            pair_vec.push(NameValuePair::new("fileName", &self.file_name));
        }

        if self.file_extension.len() > 0 {
            pair_vec.push(NameValuePair::new("fileExtension", &self.file_extension));
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

    /// Will look into the Meta Data and generate an UPnp Compailant
    /// XML String with all Data that has a Value.
    pub fn generate_upnp_xml(&self) -> String {
        let mut xml: String = String::new();

        if self.genre.len() > 0 {
            xml.push_str(&format!(
                "&lt;upnp:genre&gt;{}&lt;/upnp:genre&gt;",
                self.genre
            ));
        }

        if self.description.len() > 0 {
            xml.push_str(&format!(
                "&lt;dc:description&gt;{}&lt;/dc:description&gt;",
                self.description
            ));
        }

        if self.long_description.len() > 0 {
            xml.push_str(&format!(
                "&lt;upnp:longDescription&gt;{}&lt;/upnp:longDescription&gt;",
                self.long_description
            ));
        }

        if self.producer.len() > 0 {
            xml.push_str(&format!(
                "&lt;upnp:producer&gt;{}&lt;/upnp:producer&gt;",
                self.producer
            ));
        }

        if self.rating.len() > 0 {
            xml.push_str(&format!(
                "&lt;upnp:rating&gt;{}&lt;/upnp:rating&gt;",
                self.rating
            ));
        }

        if self.actor.len() > 0 {
            xml.push_str(&format!(
                "&lt;upnp:actor&gt;{}&lt;/upnp:actor&gt;",
                self.actor
            ));
        }

        if self.director.len() > 0 {
            xml.push_str(&format!(
                "&lt;upnp:director&gt;{}&lt;/upnp:director&gt;",
                self.director
            ));
        }

        if self.publisher.len() > 0 {
            xml.push_str(&format!(
                "&lt;dc:publisher&gt;{}&lt;/dc:publisher&gt;",
                self.publisher
            ));
        }

        if self.album.len() > 0 {
            xml.push_str(&format!(
                "&lt;upnp:album&gt;{}&lt;/upnp:album&gt;",
                self.album
            ));
        }

        if self.track_number.len() > 0 {
            xml.push_str(&format!(
                "&lt;upnp:originalTrackNumber&gt;{}&lt;/upnp:originalTrackNumber&gt;",
                self.track_number
            ));
        }

        if self.playlist.len() > 0 {
            xml.push_str(&format!(
                "&lt;upnp:playlist&gt;{}&lt;/upnp:playlist&gt;",
                self.playlist
            ));
        }

        if self.contributor.len() > 0 {
            xml.push_str(&format!(
                "&lt;dc:contributor&gt;{}&lt;/upnp:contributor&gt;",
                self.contributor
            ));
        }

        if self.date.len() > 0 {
            xml.push_str(&format!("&lt;upnp:date&gt;{}&lt;/upnp:date&gt;", self.date));
        }

        for language in &self.languages {
            xml.push_str(&format!(
                "&lt;dc_language&gt;{}&lt;/dc:language&gt;",
                language
            ));
        }

        for artist in &self.artists {
            xml.push_str(&format!(
                "&lt;upnp:artist&gt;{}&lt;/upnp:artist&gt;",
                artist
            ));
        }

        for copyright in &self.copyrights {
            xml.push_str(&format!(
                "&lt;dc:rights&gt;{}&lt;/dc:rightse&gt;",
                copyright
            ));
        }

        xml
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
#[derive(Clone)]
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
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the Meta Data
    /// * `value` - Value of the Meta Data
    pub fn insert_meta_data(&mut self, name: &str, value: &str) {
        match name {
            "title" => self.meta_data.title = value.to_string(),
            "fileName" => self.meta_data.file_name = value.to_string(),
            "fileExtension" => self.meta_data.file_extension = value.to_string(),
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

    /// Will generate the UPnP XML String representing this Media Item containing
    /// all available Values.
    ///
    /// # Arguments
    ///
    /// * `renderer_cfg` - Configuration for the Renderer that should be served
    /// * `server_cfg` - The Servers Configuration
    pub fn generate_upnp_xml(
        &self,
        renderer_cfg: &RendererConfiguration,
        server_cfg: &ServerConfiguration,
    ) -> String {
        // Generat first set of Parameters
        let mut title: String = self.meta_data.file_name.clone();
        let mut xml: String = format!(
            "&lt;item id=\"{}\" parentID=\"{}\" restricted=\"1\"&gt;",
            self.id,
            self.parent_id
        );

        // Add title how it is configured
        if renderer_cfg.title_instead_of_name {
            title = self.meta_data.title.clone();
        } else {
            if renderer_cfg.hide_file_extension == false {
                title.push_str(".");
                title.push_str(&self.meta_data.file_extension);
            }

        }

        // Replace some special chars to avoid some renderer problems
        title = title.replace("&amp;", " u. ").replace("&", " u. ");

        // Add title,res String and Mime Type
        xml.push_str(&format!("&lt;dc:title&gt;{}&lt;/dc:title&gt;&lt;res xmlns:dlna=\"urn:schemas-dlna-org:metadata-1-0/\" protocolInfo=\"http-get:*:{}:DLNA.ORG_OP=11;DLNA.ORG_CI=0\" ", title, self.get_mime_type()));

        // Add MediaType specifig Values
        match self.media_type {
            MediaType::PICTURE => {
                xml.push_str(&format!(
                    "bitrate=\"{}\" duration=\"{}\" nrAudioChannels=\"{}\" sampleFrequency=\"{}\" ",
                    self.get_bitrate(),
                    self.duration,
                    self.get_audio_channels(),
                    self.get_sample_rate()
                ));

                match self.media_type {
                    MediaType::VIDEO => {
                        xml.push_str(&format!("resolution=\"{}\" ", self.get_resolution()));
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        // Add Values always required
        xml.push_str(
            &format!("size=\"{}\"&gt;http://{}:{}/stream/{}&lt;/res&gt;",
            self.file_size,
            server_cfg.server_ip,
            server_cfg.server_port,
            self.id,
            ),
        );

        // Add UPnP Media Type
        match self.media_type {
            MediaType::UNKNOWN => {
                xml.push_str("&lt;upnp:class&gt;object.item.imageItem&lt;/upnp:class&gt;")
            }
            MediaType::AUDIO => {
                xml.push_str("&lt;upnp:class&gt;object.item.audioItem&lt;/upnp:class&gt;")
            }
            MediaType::PICTURE => {
                xml.push_str("&lt;upnp:class&gt;object.item.imageItem&lt;/upnp:class&gt;")
            }
            MediaType::VIDEO => {
                xml.push_str("&lt;upnp:class&gt;object.item.videoItem&lt;/upnp:class&gt;")
            }
        }

        // Add additional Meta Data and close Tag
        xml.push_str(&format!(
            "{}&lt;/item&gt;",
            self.meta_data.generate_upnp_xml()
        ));

        xml
    }

    /// Calculates the total Bitrate this Item provides summing each Streams individual Bitrate
    fn get_bitrate(&self) -> u64 {
        let mut rate: u64 = 0;

        for stream in &self.media_tracks {
            rate += stream.bitrate;
        }

        rate
    }

    /// Returns the Number of Audio Channels of the default Audio Stream
    ///
    /// # TO-DO
    /// - Match with the requested Stream instead of default one
    fn get_audio_channels(&self) -> u8 {
        for stream in &self.media_tracks {
            if stream.audio_channels != 0 && stream.is_default {
                return stream.audio_channels;
            }
        }

        0
    }

    /// Returns the Sample Rate in Hz of the default Audio Stream
    ///
    /// # TO-DO
    /// - Match with the requested Stream instead of default one
    fn get_sample_rate(&self) -> u32 {
        for stream in &self.media_tracks {
            if stream.sample_rate != 0 && stream.is_default {
                return stream.sample_rate;
            }
        }

        0
    }

    /// Returns the Resulution as WidthxHeight String for the default Video Stream
    ///
    /// # TO-DO
    /// - Match with the requested Stream instead default one
    fn get_resolution(&self) -> String {
        for stream in &self.media_tracks {
            if stream.frame_width != 0 && stream.frame_height != 0 && stream.is_default {
                return format!("{}x{}", stream.frame_width, stream.frame_height.to_string());
            }
        }

        String::new()
    }

    /// Returns the Mime Type used for this Item
    ///
    /// # TO-DO
    /// Use the Format Containers to determine Mime Type instead of static lists
    pub fn get_mime_type(&self) -> String {
        match self.media_type {
            MediaType::VIDEO => {
                match self.meta_data.file_extension.to_lowercase().as_str() {
                    "mkv" => return "video/x-matroska".to_string(),
                    "avi" => return "video/x-msvideo".to_string(),
                    "mpeg" | "mpg" | "mpe" => return "video/mpeg".to_string(),
                    "mov" | "qt" => return "video/quicktime".to_string(),
                    "mp4" => return "video/mp4".to_string(),
                    _ => return "video/*".to_string(),

                }
            }
            MediaType::AUDIO => {
                match self.meta_data.file_extension.to_lowercase().as_str() {
                    "mp3" => return "audio/mpeg".to_string(),
                    "wav" => return "audio/x-wav".to_string(),
                    "flac" => return "audio/flac".to_string(),
                    _ => return "audio/*".to_string(),
                }
            }
            MediaType::PICTURE => {
                match self.meta_data.file_extension.to_lowercase().as_str() {
                    "jpg" | "jpeg" | "jpe" => return "image/jpeg".to_string(),
                    "png" => return "image/png".to_string(),
                    _ => return "image/*".to_string(),
                }
            }
            _ => return "*".to_string(),
        }

    }
}

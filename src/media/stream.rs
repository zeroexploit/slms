use tools;

/// # StreamType
///
/// Enumaration that provides all possibilities of what
/// kind of Stream we got.
/// E.g.: Audio Track, Subtitle or Video Track
pub enum StreamType {
    UNKNOWN,
    AUDIO,
    VIDEO,
    IMAGE,
    SUBTITLE,
}

impl StreamType {
    /// Convert the enumaration Type to a number returned as a String
    ///
    /// In Order to store the StreamType (along with other Data) in
    /// a XML File (aka the Media Database) we need to assign a
    /// printable Value to each StreamType. And as we need a String
    /// for the NameValuePairs we directly convert it.
    ///
    /// # Assigned Values
    ///
    /// `StreamType::UNKNOWN` - "0"
    /// `StreamType::AUDIO` - "1"
    /// `StreamType::VIDEO` - "2"
    /// `StreamType::IMAGE` - "3"
    /// `StreamType::SUBTITLE` - "4"
    pub fn to_string(&self) -> String {
        match *self {
            StreamType::AUDIO => "1".to_string(),
            StreamType::VIDEO => "2".to_string(),
            StreamType::IMAGE => "3".to_string(),
            StreamType::SUBTITLE => "4".to_string(),
            _ => "0".to_string(),
        }
    }

    /// Convert Strings back to the enumeration Type.
    /// This is used to parse the XML Data back
    pub fn from_string(content: &str) -> StreamType {
        match content {
            "1" => StreamType::AUDIO,
            "2" => StreamType::VIDEO,
            "3" => StreamType::IMAGE,
            "4" => StreamType::SUBTITLE,
            _ => StreamType::UNKNOWN,
        }
    }
}

/// # Stream
///
/// As there might be more then one Video and / or Audio Track
/// inside a File we need to make sure we can access each Stream separatly
/// depending on what the Renderers need.
///
/// The Stream struct stores all Information required for a single Media
/// Track inside a File. Including Codec, Bitrate, Type, Language etc.
/// Combining these stream Objects in Media Item we got complete Access on
/// every Media a File might provide and use that to feed a Renderer as it is
/// configurated.
///
/// As UPnP defines what kind of Information can be send about a Media File, we
/// do not store all Information a Media Track might contain. No Meta Data like
/// "Encoder" or similar. There is no Point in storing that information. Better
/// save some memory here.
pub struct Stream {
    pub index: u8,
    pub stream_type: StreamType,
    pub codec_name: String,
    pub bitrate: u64,
    pub audio_channels: u8,
    pub sample_rate: u32,
    pub frame_width: u16,
    pub frame_height: u16,
    pub bit_depth: u8,
    pub language: String,
    pub is_default: bool,
    pub is_forced: bool,
}

impl Stream {
    /// This creates an empty Stream Object that is set to
    /// an unknown Type and all other Values to Zero. As in this State it is ready to
    /// be set to any kind of Media Track but can not be used directly without beeing
    /// served with Information.
    pub fn new() -> Stream {
        Stream {
            index: 0,
            stream_type: StreamType::UNKNOWN,
            codec_name: String::new(),
            bitrate: 0,
            audio_channels: 0,
            sample_rate: 0,
            frame_width: 0,
            frame_height: 0,
            bit_depth: 0,
            language: String::new(),
            is_default: false,
            is_forced: false,
        }
    }

    /// Convert the Attributes of this Object into a List of Name Value Pairs
    ///
    /// This is done in Order to prepare the Values to be stored inside the
    /// XML Media Database.
    pub fn get_name_value_pairs(&self) -> Vec<tools::NameValuePair> {
        let pair_vec: Vec<tools::NameValuePair> =
            vec![
                tools::NameValuePair::new("index", &self.index.to_string()),
                tools::NameValuePair::new("type", &self.stream_type.to_string()),
                tools::NameValuePair::new("codecName", &self.codec_name),
                tools::NameValuePair::new("bitrate", &self.bitrate.to_string()),
                tools::NameValuePair::new("nrAudioChannels", &self.audio_channels.to_string()),
                tools::NameValuePair::new("sampleFrequenzy", &self.sample_rate.to_string()),
                tools::NameValuePair::new("width", &self.frame_width.to_string()),
                tools::NameValuePair::new("height", &self.frame_height.to_string()),
                tools::NameValuePair::new("bitDepth", &self.bit_depth.to_string()),
                tools::NameValuePair::new("language", &self.language),
                tools::NameValuePair::new("isDefault", &self.is_default.to_string()),
                tools::NameValuePair::new("isForced", &self.is_forced.to_string()),
            ];

        pair_vec
    }
}

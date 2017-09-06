/// # SourceTargetMap
///
/// This is an universal Structure that holds two Strings.
/// One as source of something that needs to be converted or
/// is related to the seconde one the target. Can be used
/// whenever it is needed.
pub struct SourceTargetMap {
    pub source: String,
    pub target: String,
}

impl SourceTargetMap {
    /// Creates a new and empty SourceTargetMap
    pub fn new() -> SourceTargetMap {
        SourceTargetMap {
            source: String::new(),
            target: String::new(),
        }
    }
}

/// # RendererConfiguration
///
/// This structure stores all Configurations Settings that apply to
/// a single Renderer. As the projects target is to ensure every
/// device can be configured individually, here are most of the
/// relevant settings located.
pub struct RendererConfiguration {
    pub display_name: String, // Name of the Renderer as it appears in Logs
    pub user_agent_search: Vec<String>, // Text in "User-Agent" Header to search for and identifiy a Device
    pub remote_ip: String, // Ip to identify a Device by
    pub file_extensions: Vec<String>, // List of supported File Extensions by the Device
    pub container_maps: Vec<SourceTargetMap>, // List of what unsupported Container should be mapped to what kind of supported one
    pub transcode_container: String, // Default Container for transcoded elements
    pub audio_channels: u8, // Number of Audio Channels the device supports
    pub transcode_enabled: bool, // Enable Transcoding Engine in general?
    pub transcode_audio_enabled: bool, // Enable transcoding of Audio Streams?
    pub transcode_video_enabled: bool, // Enable transcoding of Video Streams?
    pub transcode_codecs: Vec<SourceTargetMap>, // Mappings what codec should be transcoded into what other codec
    pub audio_languages: Vec<String>, // List of Audio Languages to play on this device
    pub subtitle_connection: Vec<SourceTargetMap>, // Connection what subtitle language to use for what audio language
    pub encode_subtitles: bool, // Encode Subtitels into Video Stream insted of providing an individual Sub Track?
    pub title_instead_of_name: bool, // Use Meta-Data Title instead of File Name?
    pub hide_file_extension: bool, // Hide the File Extension from the user?
    pub mux_to_match: bool, // Remove any unneeded Tracks (and provide a single audio and video track) instead of all?
}

impl RendererConfiguration {
    /// Creates a new RendererConfiguration Structure and inserts some default
    /// Values. Should not be used directly.
    pub fn new() -> RendererConfiguration {
        RendererConfiguration {
            display_name: String::from("DEFAULT"),
            user_agent_search: Vec::new(),
            remote_ip: String::new(),
            file_extensions: Vec::new(),
            container_maps: Vec::new(),
            transcode_container: String::new(),
            audio_channels: 2,
            transcode_enabled: false,
            transcode_audio_enabled: false,
            transcode_video_enabled: false,
            transcode_codecs: Vec::new(),
            audio_languages: Vec::new(),
            subtitle_connection: Vec::new(),
            encode_subtitles: false,
            title_instead_of_name: false,
            hide_file_extension: false,
            mux_to_match: false,
        }
    }
}

use media::item::Item;
use std::process::Command;
use tools::XMLParser;
use tools::XMLEntry;
use media::Stream;
use media::StreamType;
use std::fs;
use std::time;

/// # MediaParser
///
/// This is an empty Structure providing all functionality
/// to open Media Files, read in their Meta Information and
/// create Item structures out of it and make them available
/// to the Media Server.
///
/// # To-Do
/// I can't compile ffmpeg in order to use it directly in rust.
/// So once i made that work the parser should use it instead
/// of calling an external tool.
pub struct MediaParser {}

impl MediaParser {
    /// Create a new MediaParser Structure
    pub fn new() -> MediaParser {
        MediaParser {}
    }

    /// Parse the given File into an Item structure. Returns
    /// true if File was successfull parsed. False if not.
    /// This function uses ffprobe in order to obtain the
    /// Media Information. -> Needs to be changed in future.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the File to parse
    /// * `target` - Referenze to a Item Structure to hold the Data
    pub fn parse_file(&self, path: &str, target: &mut Item) -> bool {

        let output = Command::new("ffprobe")
            .args(
                &[
                    "-v",
                    "quiet",
                    "-print_format",
                    "xml",
                    "-show_format",
                    "-show_streams",
                    "-unit",
                    path,
                ],
            )
            .output()
            .expect("failed to execute process");

        // Convert Output to String
        let xml_out: String = String::from_utf8(output.stdout).expect("Can not convert!");

        // Check if  everything we need is there
        if xml_out.len() == 0 || xml_out.find("format").is_none() ||
            xml_out.find("streams").is_none()
        {
            return false;
        }

        target.file_path = path.to_string();

        // Parse the XML Output
        let xml_parser: XMLParser = XMLParser::open(&xml_out);

        // Parse the Format
        let format_entry: XMLEntry = XMLParser::find_tag(&xml_parser.xml_entries, "format");

        target.format_container.add_file_extension(&path.split(".")
            .last()
            .unwrap()
            .to_lowercase());

        for attr in format_entry.attributes {
            match attr.name.as_ref() {
                "format_name" => target.format_container.name = attr.value,
                "duration" => {
                    target.duration =
                        self.convert_duration(&attr.value[0..attr.value.find(" ").unwrap()]);
                }
                "size" => {
                    target.file_size = attr.value[0..attr.value.find(" ").unwrap()]
                        .to_string()
                        .parse::<u64>()
                        .unwrap();
                }
                _ => (),
            }

        }

        for sub_tag in format_entry.sub_tags {
            if sub_tag.tag == "tag" {
                self.insert_meta_data(
                    &sub_tag.attributes.get(0).unwrap().value,
                    &sub_tag.attributes.get(1).unwrap().value,
                    target,
                );
            }
        }

        // Parse the Streams
        let streams_entry: XMLEntry = XMLParser::find_tag(&xml_parser.xml_entries, "streams");

        for stream_entry in streams_entry.sub_tags {
            let mut stream: Stream = Stream::new();

            // Parse the Streams attributes
            for attr in stream_entry.attributes {

                match attr.name.as_ref() {
                    "index" => stream.index = attr.value.parse::<u8>().unwrap(),
                    "codec_name" => stream.codec_name = attr.value,
                    "codec_type" => {
                        match attr.value.as_ref() {
                            "audio" => stream.stream_type = StreamType::AUDIO,
                            "video" => stream.stream_type = StreamType::VIDEO,
                            "image" => stream.stream_type = StreamType::IMAGE,
                            "picture" => stream.stream_type = StreamType::IMAGE,
                            "subtitle" => stream.stream_type = StreamType::SUBTITLE,
                            _ => stream.stream_type = StreamType::UNKNOWN,
                        }
                    }
                    "width" => stream.frame_width = attr.value.parse::<u16>().unwrap(),
                    "height" => stream.frame_height = attr.value.parse::<u16>().unwrap(),
                    "bits_per_sample" => stream.bit_depth = attr.value.parse::<u8>().unwrap(),
                    "sample_rate" => {
                        stream.sample_rate = attr.value[0..attr.value.find(" ").unwrap()]
                            .to_string()
                            .parse::<u32>()
                            .unwrap()
                    }
                    "channels" => stream.audio_channels = attr.value.parse::<u8>().unwrap(),
                    "bit_rate" => {
                        stream.bitrate = attr.value[0..attr.value.find(" ").unwrap()]
                            .to_string()
                            .parse::<u64>()
                            .unwrap()
                    }
                    _ => (),
                }
            }

            // Parse the Streams disposition
            for sub_stream in stream_entry.sub_tags {
                match sub_stream.tag.as_ref() {
                    "disposition" => {
                        for attr in sub_stream.attributes {

                            match attr.name.as_ref() {
                                "default" => {
                                    if attr.value == "0" {
                                        stream.is_default = false;
                                    } else {
                                        stream.is_default = true;
                                    }
                                } 
                                "forced" => {
                                    if attr.value == "0" {
                                        stream.is_forced = false;
                                    } else {
                                        stream.is_forced = true;
                                    }
                                }
                                _ => (),
                            }
                        }
                    } 
                    "tag" => {
                        if sub_stream.attributes.get(0).unwrap().value == "language" {
                            stream.language = sub_stream.attributes.get(1).unwrap().value.clone();
                        }
                    }
                    _ => (),
                }
            }

            // Add the Stream to the Media Item
            target.media_tracks.push(stream);
        }

        // Add missing Values
        let metadata = fs::metadata(path);
        target.last_modified = metadata
            .unwrap()
            .modified()
            .unwrap()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();


        return true;
    }

    /// Takes the ffmpeg meta tags and inserts them into the MetaData structure
    /// every Item holds.
    fn insert_meta_data(&self, name: &str, value: &str, target: &mut Item) {
        match name {
            "album" => target.meta_data.album = value.to_string(),
            "artist" => target.meta_data.artists.push(value.to_string()),
            "composer" => target.meta_data.composer = value.to_string(),
            "copyright" => target.meta_data.copyrights.push(value.to_string()),
            "date" => target.meta_data.date = value.to_string(),
            "comment" => target.meta_data.description = value.to_string(),
            "genre" => target.meta_data.genre = value.to_string(),
            "language" => target.meta_data.languages.push(value.to_string()),
            "publisher" => target.meta_data.publisher = value.to_string(),
            "track" => target.meta_data.track_number = value.to_string(),
            "performer" => target.meta_data.actor = value.to_string(),
            _ => (),
        }
    }

    /// Takes the Medias Duration in seconds and converts it to
    /// hh:mm:ss.ms format. That format than is later used
    /// for UPnP.
    fn convert_duration(&self, duration: &str) -> String {
        let seconds: f64 = duration.parse::<f64>().unwrap();
        let hours: u32 = (seconds / (60.0 * 60.0)) as u32;
        let minutes: u32 = ((seconds / 60.0) - (hours as f64 * 60.0)) as u32;
        let seconds_dif: u32 = (seconds as u32 - (hours * 60 * 60) - (minutes * 60)) as u32;

        let mut result: String = String::new();

        if hours < 10 {
            result.push_str("0");
        }

        result.push_str(&hours.to_string());
        result.push_str(":");

        if minutes < 10 {
            result.push_str("0");
        }

        result.push_str(&minutes.to_string());
        result.push_str(":");

        if seconds_dif < 10 {
            result.push_str("0");
        }

        result.push_str(&seconds_dif.to_string());
        result.push_str(&duration[duration.find(".").unwrap()..].to_string()[..3]);

        result
    }
}

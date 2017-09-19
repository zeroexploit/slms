use std::process::Command;
use std::{fs, time};
use std::path::Path;

use media::item::Item;
use tools::{XMLParser, XMLEntry};
use media::{Stream, StreamType, MediaType};

/// # MediaParser
///
/// This is an moule designed to providing all functionality
/// to open Media Files, read in their Meta Information and
/// create Item structures out of it and make them available
/// to the Media Server.
///
/// # To-Do
/// I can't compile ffmpeg in order to use it directly in rust.
/// So once i made that work the parser should use it instead
/// of calling an external tool.
///
/// Parse the given File into an Item structure. Returns
/// true if File was successfull parsed. False if not.
/// This function uses ffprobe in order to obtain the
/// Media Information. -> Needs to be changed in future.
///
/// # Arguments
///
/// * `path` - Path to the File to parse
/// * `target` - Referenze to a Item Structure to hold the Data
pub fn parse_file(path: &str, target: &mut Item) -> bool {

    let output = match Command::new("ffprobe")
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
        .output() {
        Ok(value) => value,
        Err(_) => return false,
    };

    // Convert Output to String
    let xml_out: String = match String::from_utf8(output.stdout) {
        Ok(value) => value,
        Err(_) => return false,
    };

    // Check if  everything we need is there
    if xml_out.len() == 0 || xml_out.find("format").is_none() || xml_out.find("streams").is_none() {
        return false;
    }

    target.file_path = path.to_string();

    // Parse the XML Output
    let xml_parser: XMLParser = XMLParser::open(&xml_out);

    // Parse the Format
    let format_entry: XMLEntry = XMLParser::find_tag(&xml_parser.xml_entries, "format");

    // Set File Extension
    let file_extension: String = match path.split(".").last() {
        Some(value) => value.to_lowercase(),
        None => return false,
    };

    target.format_container.add_file_extension(&file_extension);

    // Get the Format Attributes
    for attr in format_entry.attributes {
        match attr.name.as_ref() {
            "format_name" => target.format_container.name = attr.value,
            "duration" => {
                let duration = attr.value[..match attr.value.find(" ") {
                                              Some(value) => value,
                                              None => continue,
                                          }].to_string();
                target.duration = convert_duration(&duration);
            }
            "size" => {
                let file_size = attr.value[..match attr.value.find(" ") {
                                               Some(value) => value,
                                               None => continue,
                                           }].to_string();
                target.file_size = match file_size.parse::<u64>() {
                    Ok(value) => value,
                    Err(_) => continue,
                };
            }
            _ => (),
        }

    }

    // Get the Format Tags
    for sub_tag in format_entry.sub_tags {
        if sub_tag.tag == "tag" {
            let tag_name = match sub_tag.attributes.get(0) {
                Some(value) => value,
                None => continue,
            };
            let tag_value = match sub_tag.attributes.get(1) {
                Some(value) => value,
                None => continue,
            };
            insert_meta_data(&tag_name.value, &tag_value.value, target);
        }
    }

    // Parse the Streams
    let streams_entry: XMLEntry = XMLParser::find_tag(&xml_parser.xml_entries, "streams");
    let mut has_audio = false;
    let mut has_video = false;

    for stream_entry in streams_entry.sub_tags {
        let mut stream: Stream = Stream::new();

        // Parse the Streams attributes
        for attr in stream_entry.attributes {

            match attr.name.as_ref() {
                "index" => {
                    stream.index = match attr.value.parse::<u8>() {
                        Ok(value) => value,
                        Err(_) => continue,
                    }
                }
                "codec_name" => stream.codec_name = attr.value,
                "codec_type" => {
                    match attr.value.as_ref() {
                        "audio" => {
                            has_audio = true;
                            stream.stream_type = StreamType::AUDIO;
                        }
                        "video" => {
                            has_video = true;
                            stream.stream_type = StreamType::VIDEO
                        }
                        "image" => stream.stream_type = StreamType::IMAGE,
                        "picture" => stream.stream_type = StreamType::IMAGE,
                        "subtitle" => stream.stream_type = StreamType::SUBTITLE,
                        _ => stream.stream_type = StreamType::UNKNOWN,
                    }
                }
                "width" => {
                    stream.frame_width = match attr.value.parse::<u16>() {
                        Ok(value) => value,
                        Err(_) => continue,
                    }
                }
                "height" => {
                    stream.frame_height = match attr.value.parse::<u16>() {
                        Ok(value) => value,
                        Err(_) => continue,
                    }
                }
                "bits_per_sample" => {
                    stream.bit_depth = match attr.value.parse::<u8>() {
                        Ok(value) => value,
                        Err(_) => continue,
                    }
                }
                "sample_rate" => {
                    stream.sample_rate = match attr.value[..match attr.value.find(" ") {
                                                                Some(value) => value,
                                                                None => continue,
                                                            }].to_string()
                        .parse::<u32>() {
                        Ok(value) => value,
                        Err(_) => continue,
                    }
                }
                "channels" => {
                    stream.audio_channels = match attr.value.parse::<u8>() {
                        Ok(value) => value,
                        Err(_) => continue,
                    }
                }
                "bit_rate" => {
                    stream.bitrate = match attr.value[..match attr.value.find(" ") {
                                                            Some(value) => value,
                                                            None => continue,
                                                        }].to_string()
                        .parse::<u64>() {
                        Ok(value) => value,
                        Err(_) => continue,
                    }
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
                    let tag = match sub_stream.attributes.get(0) {
                        Some(value) => value,
                        None => continue,
                    };
                    if tag.value == "language" {
                        let tag_value = match sub_stream.attributes.get(1) {
                            Some(value) => value,
                            None => continue,
                        };
                        stream.language = tag_value.value.clone();
                    }
                }
                _ => (),
            }
        }

        // Add the Stream to the Media Item if not unknown
        match stream.stream_type { 
            StreamType::UNKNOWN => {}
            _ => {
                target.media_tracks.push(stream);
            }
        }
    }

    // Add last modified date
    let metadata = fs::metadata(path);
    target.last_modified = match metadata {
        Ok(some) => {
            match some.modified() {
                Ok(in_some) => {
                    match in_some.duration_since(time::UNIX_EPOCH) {
                        Ok(in_in_some) => in_in_some.as_secs(),
                        Err(_) => {
                            return false;
                        }
                    }
                }
                Err(_) => {
                    return false;
                }
            }
        }
        Err(_) => {
            return false;
        }
    };

    // Add Filename and Extension
    let f_path = Path::new(path);
    target.meta_data.file_extension = match f_path.extension() {
        Some(value) => {
            match value.to_str() {
                Some(i_value) => i_value.to_string(),
                None => return false,
            }
        }
        None => return false,
    };
    target.meta_data.file_name = match f_path.file_name() {
        Some(value) => {
            match value.to_str() {
                Some(i_value) => i_value.to_string(),
                None => return false,
            }
        }
        None => return false,
    };
    target.meta_data.file_name =
        target.meta_data.file_name[..(target.meta_data.file_name.len() -
                                          target.meta_data.file_extension.len() -
                                          1)]
            .to_string();

    // Determine Media Type
    if has_audio || has_video {
        if has_audio {
            target.media_type = MediaType::AUDIO;
        }

        if has_video {
            target.media_type = MediaType::VIDEO;
        }
    } else {
        if has_video == false && has_audio == false && target.media_tracks.len() > 0 {
            target.media_type = MediaType::PICTURE;
        } else {
            target.media_type = MediaType::UNKNOWN;
            return false;
        }
    }

    return true;
}

/// Takes the ffmpeg meta tags and inserts them into the MetaData structure
/// every Item holds.
///
/// # Arguments
///
/// * `name` - Name of the Meta Tag
/// * `value` - Value of the Meta Tag
/// * `target` - Item to store the Value in
fn insert_meta_data(name: &str, value: &str, target: &mut Item) {
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
        "title" => target.meta_data.title = value.to_string(),
        _ => (),
    }
}

/// Takes the Medias Duration in seconds and converts it to
/// hh:mm:ss.ms format. That format than is later used
/// for UPnP.
///
/// # Arguments
///
/// * `duration` - Duration as Second String in sssss.ms format
fn convert_duration(duration: &str) -> String {
    let seconds: f64 = match duration.parse::<f64>() {
        Ok(value) => value,
        Err(_) => 0.0,
    };
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

    let ms = match duration.find(".") {
        Some(value) => duration[value..].to_string(),
        None => ".00".to_string(),
    };

    result.push_str(&ms[..3]);

    result
}

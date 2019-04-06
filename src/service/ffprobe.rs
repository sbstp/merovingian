use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::mero::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct MediaInfo {
    pub video: Vec<VideoStream>,
    pub audio: Vec<AudioStream>,
    pub subtitle: Vec<SubtitleStream>,
}

pub fn scan(path: impl AsRef<Path>) -> Result<MediaInfo> {
    // ffprobe -v quiet -print_format json -show_streams <path>
    let output = Command::new("ffprobe")
        .args(&["-v", "quiet", "-print_format", "json", "-show_streams"])
        .arg(path.as_ref())
        .output()?;

    if !output.status.success() {
        return Err(Error::SpawnError(String::from_utf8_lossy(&output.stderr).into_owned()));
    }

    let probe: Probe = serde_json::from_reader(&output.stdout[..])?;
    let mut video = vec![];
    let mut audio = vec![];
    let mut subtitle = vec![];

    for stream in probe.streams.into_iter() {
        match stream {
            Stream::Video(inner) => video.push(inner),
            Stream::Audio(inner) => audio.push(inner),
            Stream::Subtitle(inner) => subtitle.push(inner),
        }
    }

    Ok(MediaInfo { video, audio, subtitle })
}

#[derive(Debug, Deserialize)]
struct Probe {
    streams: Vec<Stream>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "codec_type")]
enum Stream {
    #[serde(rename = "video")]
    Video(VideoStream),
    #[serde(rename = "audio")]
    Audio(AudioStream),
    #[serde(rename = "subtitle")]
    Subtitle(SubtitleStream),
}

#[derive(Debug, Deserialize)]
pub struct VideoStream {
    pub index: u32,
    pub codec_name: String,
    pub width: u32,
    pub height: u32,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct AudioStream {
    pub index: u32,
    pub codec_name: String,
    pub bit_rate: String,
    pub channels: u32,
    pub tags: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct SubtitleStream {
    pub index: u32,
    pub codec_name: String,
    pub tags: HashMap<String, String>,
}

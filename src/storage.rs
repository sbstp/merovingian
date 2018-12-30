use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mero::{NonNan, Result, SubtitleFile};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    library: PathBuf,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Config> {
        let reader = BufReader::new(File::open(path.as_ref())?);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    pub fn save(path: impl AsRef<Path>, config: &Config) -> Result<()> {
        let writer = BufWriter::new(File::create(path.as_ref())?);
        serde_json::to_writer_pretty(writer, config)?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subtitle {
    pub path: PathBuf,
    pub lang: String,
    pub ext: String,
}

impl From<SubtitleFile> for Subtitle {
    fn from(sub: SubtitleFile) -> Subtitle {
        Subtitle {
            path: sub.file.path().to_owned(),
            lang: sub.lang().to_owned(),
            ext: sub.ext().to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Match {
    pub path: PathBuf,
    pub fingerprint: String,
    pub score: NonNan,
    pub title_id: u32,
    pub subtitles: Vec<Subtitle>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Scan {
    /// New matches found
    pub matches: Vec<Match>,
    /// Paths ignored because they are already in the database
    pub ignored: Vec<PathBuf>,
    /// Paths that were detected as duplicates
    pub duplicates: Vec<PathBuf>,
    /// Paths that did not match anything
    pub unmatched: Vec<PathBuf>,
}

impl Scan {
    pub fn new() -> Scan {
        Scan {
            matches: vec![],
            ignored: vec![],
            duplicates: vec![],
            unmatched: vec![],
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Scan> {
        let reader = BufReader::new(File::open(path.as_ref())?);
        let scan = bincode::deserialize_from(reader)?;
        Ok(scan)
    }

    pub fn save(path: impl AsRef<Path>, scan: &Scan) -> Result<()> {
        let writer = BufWriter::new(File::create(path.as_ref())?);
        bincode::serialize_into(writer, scan)?;
        Ok(())
    }
}

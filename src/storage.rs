use std::env;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::mero::{Fingerprint, NonNan, Result, SubtitleFile};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub root_path: PathBuf,
}

lazy_static! {
    static ref CONFIG_PATH: PathBuf = {
        let home = env::var("HOME").expect("HOME variable unset");
        let home_dir = Path::new(&home);
        home_dir.join(".config/mero/config.json")
    };
}

impl Config {
    pub fn new(root_path: impl Into<PathBuf>) -> Config {
        Config {
            root_path: root_path.into(),
        }
    }

    pub fn open() -> Result<Option<Config>> {
        if CONFIG_PATH.exists() {
            Ok(Some(Config::load(CONFIG_PATH.as_path())?))
        } else {
            Ok(None)
        }
    }

    fn load(path: impl AsRef<Path>) -> Result<Config> {
        let reader = BufReader::new(File::open(path.as_ref())?);
        let config = serde_json::from_reader(reader)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        DirBuilder::new()
            .recursive(true)
            .create(CONFIG_PATH.parent().expect("CONFIG_PATH has no parent"))?;
        let writer = BufWriter::new(File::create(CONFIG_PATH.as_path())?);
        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn meta_dir(&self) -> PathBuf {
        self.root_path.join(".mero")
    }

    pub fn library_path(&self) -> PathBuf {
        self.root_path.join(".mero/library.json")
    }

    pub fn index_path(&self) -> PathBuf {
        self.root_path.join(".mero/index.bin.gz")
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
    pub fingerprint: Fingerprint,
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

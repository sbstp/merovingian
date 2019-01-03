use std::env;
use std::fs::{DirBuilder, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::mero::{MovieFile, Result};

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
pub struct Report {
    pub path: PathBuf,
    pub movies: Vec<MovieFile>,
}

impl Report {
    pub fn new(path: impl Into<PathBuf>) -> Report {
        Report {
            movies: vec![],
            path: path.into(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Report> {
        let reader = BufReader::new(File::open(path.as_ref())?);
        let scan = bincode::deserialize_from(reader)?;
        Ok(scan)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let writer = BufWriter::new(File::create(path.as_ref())?);
        bincode::serialize_into(writer, self)?;
        Ok(())
    }
}

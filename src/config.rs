use std::env;
use std::fs::DirBuilder;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::mero::{utils, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub root_path: PathBuf,
}

lazy_static! {
    static ref CONFIG_DIR: PathBuf = {
        let home = env::var("HOME").expect("HOME variable unset");
        let home_dir = Path::new(&home);
        home_dir.join(".config/mero")
    };
    static ref CONFIG_PATH: PathBuf = CONFIG_DIR.join("config.json");
}

impl Config {
    pub fn new(root_path: impl Into<PathBuf>) -> Config {
        Config {
            root_path: root_path.into(),
        }
    }

    pub fn open() -> Result<Option<Config>> {
        if CONFIG_PATH.exists() {
            Ok(Some(utils::deserialize_json(CONFIG_PATH.as_path())?))
        } else {
            Ok(None)
        }
    }

    pub fn save(&self) -> Result<()> {
        DirBuilder::new().recursive(true).create(CONFIG_DIR.as_path())?;
        utils::serialize_json(CONFIG_PATH.as_path(), self)
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn meta_dir(&self) -> PathBuf {
        self.root_path.join(".mero")
    }

    pub fn library_path(&self) -> PathBuf {
        self.root_path.join(".mero/library.bin.gz")
    }

    pub fn index_path(&self) -> PathBuf {
        self.root_path.join(".mero/index.bin.gz")
    }

    pub fn tmdb_cache_path(&self) -> PathBuf {
        self.root_path.join(".mero/tmdb-cache.bin.gz")
    }
}

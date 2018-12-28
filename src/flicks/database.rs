use std::fs::File;
use std::io;
use std::path::PathBuf;

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::error::Result;

#[derive(Deserialize, Serialize)]
pub struct Entry {
    imdb_id: u32,
    movie_file: PathBuf,
    subtitle_files: Vec<PathBuf>,
    primary_title: String,
    fingerprint: String,
}

#[derive(Deserialize, Serialize)]
pub struct Content {
    root: PathBuf,
    entries: Vec<Entry>,
}

pub struct Database {
    path: PathBuf,
    fingerprints: HashMap<String, usize>,
    content: Content,
}

impl Database {
    pub fn create(path: impl Into<PathBuf>, root: impl Into<PathBuf>) -> Database {
        Database {
            path: path.into(),
            fingerprints: HashMap::new(),
            content: Content {
                root: root.into(),
                entries: vec![],
            },
        }
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<Database> {
        let path = path.into();
        let file = File::open(&path)?;
        let db = Database {
            path: path.into(),
            fingerprints: HashMap::new(),
            content: serde_json::from_reader(file)?,
        };
        Ok(db)
    }

    pub fn commit(&self) -> Result<()> {
        let file = File::create(&self.path)?;
        serde_json::to_writer_pretty(file, &self.content)?;
        Ok(())
    }

    fn build_index(&mut self) {
        for (index, entry) in self.content.entries.iter().enumerate() {
            self.fingerprints.insert(entry.fingerprint.clone(), index);
        }
    }
}

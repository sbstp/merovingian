use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::error::Result;
use super::index;

#[derive(Deserialize, Serialize)]
pub struct Entry {
    title_id: u32,
    movie_file: PathBuf,
    subtitle_files: Vec<PathBuf>,
    fingerprint: String,
}

#[derive(Deserialize, Serialize)]
pub struct Content {
    root: PathBuf,
    entries: Vec<Entry>,
}

pub struct Library {
    path: PathBuf,
    content: Content,
}

impl Library {
    pub fn create(path: impl Into<PathBuf>, root: impl Into<PathBuf>) -> Library {
        Library {
            path: path.into(),
            content: Content {
                root: root.into(),
                entries: vec![],
            },
        }
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<Library> {
        let path = path.into();
        let file = BufReader::new(File::open(&path)?);
        let db = Library {
            path: path.into(),
            content: serde_json::from_reader(file)?,
        };
        Ok(db)
    }

    pub fn commit(&self) -> Result<()> {
        let file = BufWriter::new(File::create(&self.path)?);
        serde_json::to_writer_pretty(file, &self.content)?;
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.content.root
    }

    pub fn has_fingerprint(&self, fingeprint: &str) -> bool {
        self.content.entries.iter().any(|e| e.fingerprint == fingeprint)
    }

    pub fn has_title(&self, title_id: u32) -> bool {
        self.content.entries.iter().any(|e| e.title_id == title_id)
    }

    pub fn add_entry(&mut self, entry: &index::Entry, movie_file: impl Into<PathBuf>, fingeprint: String) {
        let movie_file = movie_file.into();
        self.content.entries.push(Entry {
            title_id: entry.title_id,
            movie_file: movie_file.to_path_buf(),
            fingerprint: fingeprint,
            subtitle_files: vec![],
        });
    }
}

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{index, Fingerprint, Result};

#[derive(Deserialize, Serialize)]
pub struct Subtitle {
    pub path: PathBuf,
    pub fingerprint: Fingerprint,
}

#[derive(Deserialize, Serialize)]
pub struct Movie {
    pub title_id: u32,
    pub path: PathBuf,
    pub subtitles: Vec<Subtitle>,
    pub fingerprint: Fingerprint,
}

#[derive(Deserialize, Serialize)]
pub struct Content {
    pub movies: Vec<Movie>,
}

pub struct Library {
    pub path: PathBuf,
    pub content: Content,
}

impl Library {
    pub fn create(path: impl Into<PathBuf>) -> Library {
        Library {
            path: path.into(),
            content: Content { movies: vec![] },
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

    pub fn has_fingerprint(&self, fingeprint: &Fingerprint) -> bool {
        self.content.movies.iter().any(|e| e.fingerprint == *fingeprint)
    }

    pub fn has_title(&self, title_id: u32) -> bool {
        self.content.movies.iter().any(|e| e.title_id == title_id)
    }

    pub fn add_movie(
        &mut self,
        title: &index::Title,
        path: impl Into<PathBuf>,
        fingeprint: Fingerprint,
        subtitles: impl Into<Vec<Subtitle>>,
    ) {
        self.content.movies.push(Movie {
            title_id: title.title_id,
            path: path.into(),
            fingerprint: fingeprint,
            subtitles: subtitles.into(),
        });
    }
}

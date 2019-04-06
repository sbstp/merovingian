use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::index::Index;
use crate::local_storage::LocalStorage;
use crate::mero::{utils, walk, MovieFile, Result, Scanner, TMDB};

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
        utils::deserialize_bin_gz(path)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        utils::serialize_bin_gz(path, self)
    }
}

pub fn cmd_scan(
    import_path: impl AsRef<Path>,
    save_path: impl Into<Option<PathBuf>>,
    config: Config,
    index: &Index,
    local_storage: &LocalStorage,
) -> Result {
    let import_path = import_path.as_ref();
    println!("Scanning import path {}", import_path.display());

    let root = walk(import_path, &local_storage.ignored)?;
    let tmdb = TMDB::new(config.tmdb_cache_path());
    let mut scanner = Scanner::new(tmdb);

    let mut report = Report::new(import_path);
    report.movies = scanner.scan_movies(&root, index)?;

    let save_path = save_path.into().unwrap_or(PathBuf::from("scan-report.mero"));
    report.save(save_path)?;

    Ok(())
}

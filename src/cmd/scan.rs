use std::path::{Path, PathBuf};

use serde::{Serialize, Deserialize};

use crate::mero::{walk, Index, Result, Scanner, TMDB, utils, MovieFile};
use crate::config::{Config};

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
) -> Result {
    let import_path = import_path.as_ref();
    println!("Scanning import path {}", import_path.display());

    let root = walk(import_path)?;
    let tmdb = TMDB::new(config.tmdb_cache_path());
    let mut scanner = Scanner::new(tmdb);

    let mut report = Report::new(import_path);
    report.movies = scanner.scan_movies(&root, index)?;

    let save_path = save_path.into().unwrap_or(PathBuf::from("scan-report.mero"));
    report.save(save_path)?;

    Ok(())
}

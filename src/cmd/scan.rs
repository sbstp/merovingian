use std::path::{Path, PathBuf};

use crate::mero::{walk, Index, Result, Scanner};
use crate::storage::Report;

pub fn cmd_scan(import_path: impl AsRef<Path>, save_path: impl Into<Option<PathBuf>>, index: &Index) -> Result {
    let import_path = import_path.as_ref();
    println!("Scanning import path {}", import_path.display());

    let root = walk(import_path)?;
    let mut scanner = Scanner::new();

    let mut report = Report::new(import_path);
    report.movies = scanner.scan_movies(&root, index)?;

    let save_path = save_path.into().unwrap_or(PathBuf::from("scan-report.mero"));
    report.save(save_path)?;

    Ok(())
}

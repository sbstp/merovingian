use std::path::Path;

use crate::mero::{fingerprint, walk, Index, Library, Result, Scanner};
use crate::storage::{Match, Scan};

pub fn cmd_scan(import_path: impl AsRef<Path>, index: &Index, library: &Library) -> Result {
    let import_path = import_path.as_ref();
    println!("Scanning import path {}", import_path.display());

    let root = walk(import_path)?;
    let mut scanner = Scanner::new();
    let movies = scanner.scan_movies(&root);

    let mut scan = Scan::new();

    for movie in movies {
        let path = movie.file.path();
        let display_path = path.strip_prefix(import_path).unwrap().display().to_string();

        println!("Creating fingerprint for {}", display_path);
        let fp = fingerprint::file(path)?;

        if library.has_fingerprint(&fp) {
            println!("Already in the library {}", display_path);
            scan.ignored.push(path.to_owned());
        } else {
            if let Some(scored) = index.find(&movie.title, Some(movie.year)) {
                let title = scored.value;

                if library.has_title(title.title_id) {
                    println!("Duplicate found at {}", display_path);
                    scan.duplicates.push(path.to_owned());
                } else {
                    println!("Match '{} ({})' for {}", title.primary_title, title.year, display_path);

                    println!("Scanning subtitles for {}", movie.file.path().display());
                    let subtitles = scanner.scan_subtitles(&movie.file);

                    scan.matches.push(Match {
                        path: path.to_owned(),
                        fingerprint: fp,
                        title_id: title.title_id,
                        score: scored.score,
                        subtitles: subtitles.into_iter().map(From::from).collect(),
                    });
                }
            } else {
                println!("No match found for {}", display_path);
                scan.unmatched.push(path.to_owned());
            }
        }
    }

    Scan::save("scan.fls", &scan)?;

    Ok(())
}

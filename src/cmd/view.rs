use std::path::Path;

use crate::mero::{Index, Library, MovieFile, Result};
use crate::storage::Report;

pub struct Classified {
    pub ignored: Vec<MovieFile>,
    pub unmatched: Vec<MovieFile>,
    pub duplicates: Vec<MovieFile>,
    pub matches: Vec<MovieFile>,
}

impl Classified {
    pub fn classify(library: &Library, movies: Vec<MovieFile>) -> Classified {
        let mut ignored = vec![];
        let mut unmatched = vec![];
        let mut duplicates = vec![];
        let mut matches = vec![];

        for movie in movies {
            if let Some(title_id_scored) = movie.title_id_scored {
                if library.has_fingerprint(&movie.fingerprint) {
                    ignored.push(movie);
                } else {
                    if library.has_title(title_id_scored.value) {
                        duplicates.push(movie);
                    } else {
                        matches.push(movie);
                    }
                }
            } else {
                unmatched.push(movie);
            }
        }

        Classified {
            ignored,
            unmatched,
            duplicates,
            matches,
        }
    }
}

pub fn cmd_view(path: impl AsRef<Path>, index: &Index, library: &Library) -> Result {
    let path = path.as_ref();

    let report = Report::load(path)?;
    let mut classified = Classified::classify(library, report.movies);

    println!("Ignored");
    println!("=======");
    for movie in classified.ignored {
        println!("{}", movie.path.display());
    }
    println!();

    println!("Unmatched");
    println!("=========");
    for movie in classified.unmatched {
        println!("{}", movie.path.display());
    }
    println!();

    println!("Duplicates");
    println!("==========");
    for movie in classified.duplicates {
        println!("{}", movie.path.display());
    }
    println!();

    classified.matches.sort_by_key(|m| {
        m.title_id_scored
            .as_ref()
            .expect("score should not be None in sort")
            .score
    });

    println!("Matches");
    println!("=======");
    for movie in classified.matches {
        let title_id_scored = movie.title_id_scored.expect("score should not be None in print");
        let score = title_id_scored.score;
        let title = index.get_title(title_id_scored.value);
        println!("Path: {}", movie.path.display());
        println!("Name: {}", movie.path.file_name().and_then(|s| s.to_str()).unwrap());
        println!("Title: {}", title.primary_title);
        println!("Year: {}", title.year);
        println!("URL: https://imdb.com/title/tt{:07}/", title.title_id);
        println!("Score: {:0.3}", score);
        println!();
    }

    Ok(())
}

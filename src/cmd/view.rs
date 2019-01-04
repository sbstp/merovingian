use std::path::Path;

use hashbrown::HashMap;

use crate::mero::{Index, Library, MovieFile, Result, TitleId};
use crate::storage::Report;

pub struct Classified {
    pub ignored: Vec<MovieFile>,
    pub unmatched: Vec<MovieFile>,
    pub duplicates: Vec<MovieFile>,
    pub matches: Vec<MovieFile>,
    pub conflicts: HashMap<TitleId, Vec<MovieFile>>,
}

impl Classified {
    pub fn classify(library: &Library, movies: Vec<MovieFile>) -> Classified {
        let mut ignored = vec![];
        let mut unmatched = vec![];
        let mut duplicates = vec![];
        let mut movies_by_title = HashMap::new();

        for movie in movies {
            if let Some(title_id_scored) = movie.title_id_scored {
                if library.has_fingerprint(&movie.fingerprint) {
                    ignored.push(movie);
                } else {
                    let title_id = title_id_scored.value;

                    if library.has_title(title_id) {
                        duplicates.push(movie);
                    } else {
                        movies_by_title.entry(title_id).or_insert(Vec::new()).push(movie);
                    }
                }
            } else {
                unmatched.push(movie);
            }
        }

        let mut matches = vec![];
        let mut conflicts = HashMap::new();

        for (title_id, titles) in movies_by_title.drain() {
            if titles.len() <= 1 {
                matches.extend(titles);
            } else {
                conflicts.insert(title_id, titles);
            }
        }

        Classified {
            ignored,
            unmatched,
            duplicates,
            matches,
            conflicts,
        }
    }
}

pub fn cmd_view(path: impl AsRef<Path>, index: &Index, library: &Library) -> Result {
    let path = path.as_ref();

    let report = Report::load(path)?;
    let mut classified = Classified::classify(library, report.movies);

    println!("Ignored (files that were already imported)");
    println!("=======");
    for movie in classified.ignored {
        println!("{}", movie.path.display());
    }
    println!();

    println!("Unmatched (files that could not be matched with a movie)");
    println!("=========");
    for movie in classified.unmatched {
        println!("{}", movie.path.display());
    }
    println!();

    println!("Duplicates (different copy of a movie already in the library)");
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

    println!("Conflicts (different copies of the same movie, not yet in the library)");
    println!("=========");
    for (title_id, movies) in classified.conflicts.iter() {
        let title = index.get_title(*title_id);
        println!("Title: {}", title.primary_title);
        println!("Year: {}", title.year);
        println!("URL: https://imdb.com/title/tt{:07}/", title.title_id.full());
        for movie in movies {
            println!("Path: {}", movie.path.display());
        }
        println!();
    }

    println!("Matches (movies to be imported)");
    println!("=======");
    for movie in classified.matches {
        let title_id_scored = movie.title_id_scored.expect("score should not be None in print");
        let score = title_id_scored.score;
        let title = index.get_title(title_id_scored.value);
        println!("Path: {}", movie.path.display());
        println!("Name: {}", movie.path.file_name().and_then(|s| s.to_str()).unwrap());
        println!("Title: {}", title.primary_title);
        println!("Year: {}", title.year);
        println!("URL: https://imdb.com/title/tt{:07}/", title.title_id.full());
        println!("Score: {:0.3}", score);
        println!();
    }

    Ok(())
}

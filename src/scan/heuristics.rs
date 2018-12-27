use hashbrown::HashSet;
use lazy_static::lazy_static;

use crate::index::Entry;
use crate::scan::vfs::File;

lazy_static! {
    static ref VIDEO_EXT: Vec<&'static str> =
        vec!["mkv", "mp4", "avi", "m4v", "webm", "flv", "vob", "mov", "wmv", "ogv", "ogg"];
    static ref SUBTITLE_EXT: Vec<&'static str> = vec!["srt", "sub", "idx", "usf", "smi"];
}

#[derive(Debug)]
pub struct ScanResult {
    pub movie: File,
    pub year: i32,
    pub title: String,
}

fn is_video(file: &File) -> bool {
    file.is_file()
        && file
            .ext()
            .map(|s| VIDEO_EXT.contains(&&s.to_lowercase()[..]))
            .unwrap_or(false)
}

fn token_splitter(c: char) -> bool {
    match c {
        c if c.is_whitespace() => true,
        '_' | '-' | '.' => true,
        _ => false,
    }
}

fn text_to_tokens<'t>(text: &'t str, tokens: &mut Vec<&'t str>) {
    tokens.clear();
    for token in text.split(token_splitter) {
        if !token.is_empty() {
            tokens.push(token);
        }
    }
}

fn is_year(token: &str) -> bool {
    token.len() == 4 && token.chars().all(|c| c.is_digit(10))
}

fn parse_title(stem: &str) -> Option<(String, i32)> {
    let stem = stem.to_lowercase();
    let mut tokens = Vec::new();
    text_to_tokens(&stem, &mut tokens);

    let (year_pos, year_token) = tokens.iter().enumerate().rev().find(|(_, t)| is_year(t))?;

    let title = tokens[..year_pos].join(" ");
    let year = year_token.parse().expect("invalid year");

    Some((title, year))
}

pub fn scan(root: &File) {
    let mut ignore: HashSet<File> = HashSet::new();

    let mut movies: Vec<ScanResult> = Vec::new();

    for child in root.descendants() {
        match (is_video(&child), child.stem()) {
            (true, Some(s)) => {
                if let Some((title, year)) = parse_title(s) {
                    // once we find a movie we try to look for peers that are small
                    // (usually featurettes, samples and extras) and mark them as ignored
                    if let Some(parent) = child.parent() {
                        if parent != *root {
                            let size = child.metadata().len() as f64;

                            for peer in parent.descendants() {
                                if peer.metadata().len() as f64 / size <= 0.40 {
                                    ignore.insert(peer);
                                }
                            }
                        }
                    }

                    movies.push(ScanResult {
                        title: title.to_string(),
                        year: year,
                        movie: child,
                    });
                }
            }
            _ => {}
        }
    }

    movies.retain(|sr| {
        let x = ignore.contains(&sr.movie);
        if x {
            println!("-: {:#?}", sr);
        }
        !x
    });

    println!("\n---------------------------\n");

    for sr in movies {
        println!(">: {:#?}", sr);
    }
}

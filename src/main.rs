#![allow(dead_code)]

mod flicks;
mod input;

use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::flicks::{
    error::Result,
    fingerprint,
    index::{self, Index},
    library::{self, Library},
    scan::{self, ScanResult, Scanner},
    transfer::{Manager, Transfer},
    utils::clean_path,
    vfs,
};
use crate::input::Input;

fn open_library(input: &Input) -> Library {
    match Library::open(".index/library.json") {
        Ok(lib) => lib,
        Err(_) => {
            let root = loop {
                let root = input.prompt("Where would you like to store your movie library?");
                match Path::new(&root).canonicalize() {
                    Ok(path) => {
                        if path.is_dir() {
                            break path;
                        } else {
                            println!("Not a directory");
                        }
                    }
                    Err(err) => println!("{}", err),
                }
            };
            Library::create(".index/library.json", root)
        }
    }
}

fn make_movie_path(primary_title: &str, year: u16, ext: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(clean_path(&format!("{} ({})", primary_title, year)));
    path.push(clean_path(&format!("{} ({}).{}", primary_title, year, ext)));
    path
}

fn make_subtitle_path(movie_path: &Path, subtitle_file: &scan::SubtitleFile) -> PathBuf {
    let ext = format!(
        "{}.{}",
        subtitle_file.lang.code(),
        &subtitle_file.format.get_name()[1..4]
    );
    movie_path.clone().with_extension(&ext)
}

fn print_transfer(transfer: &Transfer) {
    println!("Source: {}", transfer.src.display());
    println!("Destination: {}", transfer.dst.display());
    println!("Status: {}", transfer.status());
    println!();
}

fn lookup_entry<'i>(input: &Input, index: &'i Index) -> Option<&'i index::Entry> {
    loop {
        let name = input.prompt("Enter the name of the movie");
        if let Some(title) = index.best_match(&name, None) {
            let prompt = format!("Found \"{} ({}), is this correct?\"", title.primary_title, title.year);

            match input.choose(
                &prompt,
                &[
                    ('y', "use this title"),
                    ('n', "look for another title"),
                    ('s', "skip this file"),
                ],
                None,
            ) {
                'y' => return Some(title),
                'n' => continue,
                's' => return None,
                _ => unreachable!(),
            }
        }
    }
}

fn library_add(
    manager: &mut Manager,
    library: &mut Library,
    sr: &ScanResult,
    title: &index::Entry,
    ext: &str,
    fingerprint: String,
) -> Result<()> {
    let rel_path = make_movie_path(&title.primary_title, title.year, ext);

    let full_path = library.root().join(&rel_path);
    // add movie to the transfer manager
    manager.add_transfer(sr.movie.path(), &full_path);

    let mut subtitles: Vec<library::Subtitle> = vec![];

    for subtitle_file in sr.subtitles.iter() {
        let subtitle = library::Subtitle {
            path: make_subtitle_path(&full_path, &subtitle_file),
            fingerprint: fingerprint::file(&subtitle_file.file.path())?,
        };
        // add subtitle to the transfer manager
        manager.add_transfer(subtitle_file.file.path(), &subtitle.path);
        subtitles.push(subtitle);
    }

    let mut x = Instant::now();
    while manager.has_work() {
        manager.tick();
        if x.elapsed() > Duration::from_secs(1) {
            print_transfer(manager.current());
            x = Instant::now();
        }
    }
    println!("{:#?}", manager);

    // Add the movie file and the subtitles to the library. Always commit after making changes
    // to protect against failure.
    library.add_entry(&title, rel_path, fingerprint, subtitles);
    library.commit()?;

    Ok(())
}

fn main() -> Result<()> {
    env_logger::init();

    let input = input::Input::new();

    let mut library = open_library(&input);
    library.commit()?;

    let mut manager = Manager::new();
    print!("Loading index...");
    let index = Index::load_or_create_index(".index")?;
    println!(" done.");

    print!("Walking import path...");
    let root = vfs::walk("/home/simon/tank/movies/en")?;
    println!(" done.");

    print!("Scanning tree...");
    input.flush();
    let results = Scanner::new().scan(&root, &index);
    println!(" done.");

    for sr in results {
        let fingerprint = fingerprint::file(sr.movie.path())?;

        // Check if the movie file's fingerprint is already in the library.
        if library.has_fingerprint(&fingerprint) {
            // If the movie file is already in the library, we skip it.
            println!("Movie ignored because its fingerprint is already present in the library.");
            println!("Path: {}", sr.movie.path().display());
        } else {
            // If the movie file is not in the library, we start an import procedure.

            // Here we print the main movie file as well as the subtitles and their language.
            println!("File: {}", sr.movie.name());
            for sub in sr.subtitles.iter() {
                println!("Subtitle: [{}] {}", sub.lang.code(), sub.file.path().display());
            }

            // If no match was found automatically, we ask the user to perform a lookup for the title
            // on their own. If the user decides to skip the lookup, None is returned and we skip over
            // this file.
            let title = match sr.entry.or_else(|| {
                println!("No match found.");
                lookup_entry(&input, &index)
            }) {
                Some(title) => title,
                None => continue,
            };
            println!("Match: {} ({})", title.primary_title, title.year);

            // Ask the user what to do with this movie file.
            let answer = input.choose(
                "What do you want to do with this match?",
                &[
                    ('a', "add to library"),
                    ('l', "index lookup to change the title and then add to library"),
                    ('s', "skip file"),
                ],
                None,
            );
            match answer {
                'a' => {
                    library_add(&mut manager, &mut library, &sr, &title, sr.movie.ext(), fingerprint)?;
                }
                'l' => {
                    // Perform an index lookup before adding the movie file to the library. If the user
                    // decides to skip (None returned), we do nothing.
                    if let Some(title) = lookup_entry(&input, &index) {
                        library_add(&mut manager, &mut library, &sr, &title, sr.movie.ext(), fingerprint)?;
                    }
                }
                's' => {}
                _ => unreachable!(),
            }
        }
        println!();
    }

    library.commit()?;
    Ok(())
}

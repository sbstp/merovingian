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
    library::Library,
    scan,
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

fn print_transfer(transfer: &Transfer) {
    println!("Source: {}", transfer.src.display());
    println!("Destination: {}", transfer.dst.display());
    println!("Status: {}", transfer.status());
    println!();
}

fn lookup_entry<'i>(input: &Input, index: &'i Index) -> &'i index::Entry {
    loop {
        let name = input.prompt("Enter the name of the movie");
        if let Some(title) = index.best_match(&name, None) {
            if input.confirm(
                &format!("Found \"{} ({}), is this correct?\"", title.primary_title, title.year),
                None,
            ) {
                return title;
            }
        }
    }
}

fn library_add(library: &mut Library, title: &index::Entry, ext: &str, fingerprint: String) -> Result<()> {
    let rel_path = make_movie_path(&title.primary_title, title.year, ext);

    let full_path = library.root().join(&rel_path);

    // manager.add_transfer(sr.movie.path(), full_path);
    // let mut x = Instant::now();
    // while manager.has_work() {
    //     manager.tick();
    //     if x.elapsed() > Duration::from_secs(1) {
    //         print_transfer(manager.current());
    //         x = Instant::now();
    //     }
    // }
    // println!("{:#?}", manager);

    library.add_entry(&title, rel_path, fingerprint);
    library.commit()?;

    Ok(())
}

fn main() -> Result<()> {
    let input = input::Input::new();

    let mut library = open_library(&input);
    library.commit()?;

    let mut manager = Manager::new();
    let index = Index::load_or_create_index(".index")?;

    let root = vfs::walk("/home/simon/tank/downloads")?;

    for sr in scan::scan(&root, &index) {
        let fingerprint = fingerprint::file(sr.movie.path())?;

        if library.has_fingerprint(&fingerprint) {
            println!("Movie ignored because its fingerprint is already present in the library.");
            println!("Path: {}", sr.movie.path().display());
        } else {
            println!("File: {}", sr.movie.name().unwrap());

            let title = sr.entry.unwrap_or_else(|| {
                println!("No match found.");
                lookup_entry(&input, &index)
            });

            println!("Match: {} ({})", title.primary_title, title.year);

            let answer = input.choose(
                "What do you want to do with this match?",
                &[
                    ('a', "add to library"),
                    ('l', "index lookup to change the title"),
                    ('s', "skip file"),
                ],
                None,
            );
            match answer {
                'a' => {
                    library_add(&mut library, &title, sr.movie.ext().unwrap(), fingerprint)?;
                }
                'l' => {
                    let title = lookup_entry(&input, &index);
                    library_add(&mut library, &title, sr.movie.ext().unwrap(), fingerprint)?;
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

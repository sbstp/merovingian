use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use signal_hook::flag as signal;
use signal_hook::{SIGINT, SIGTERM};

use super::view::Classified;
use crate::cmd::scan::Report;
use crate::config::Config;
use crate::error::Result;
use crate::io::transfer::{Manager, Transfer};
use crate::library::{self, Library};
use crate::scan::{RelPath, SubtitleFile};
use crate::utils::clean_path;

fn make_movie_path(primary_title: &str, year: u16, ext: &str) -> RelPath {
    let mut path = PathBuf::new();
    let cleaned_name = clean_path(&format!("{} ({})", primary_title, year));
    let dotted_name = cleaned_name.replace(" ", ".");
    path.push(&dotted_name);
    path.push(format!("{}.{}", dotted_name, ext.to_lowercase()));
    RelPath::new(path).unwrap()
}

fn make_subtitle_path(movie_path: &Path, subtitle: &SubtitleFile) -> RelPath {
    let ext = format!("{}.{}", subtitle.lang, &subtitle.ext);
    RelPath::new(movie_path.with_extension(&ext)).unwrap()
}

fn print_transfer(transfer: &Transfer) {
    println!("Source: {}", transfer.src.display());
    println!("Destination: {}", transfer.dst.display());
    println!("Status: {}", transfer.status());
    println!();
}

lazy_static! {
    static ref QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub fn cmd_import(config: Config, path: impl AsRef<Path>, library: &mut Library) -> Result {
    signal::register(SIGINT, QUIT.clone()).expect("unable to setup SIGINT hook");
    signal::register(SIGTERM, QUIT.clone()).expect("unable to setup SIGTERM hook");

    let path = path.as_ref();

    let report = Report::load(path)?;
    let classified = Classified::classify(&library, report.movies)?;

    let mut finished = 0;
    let len = classified.matches.len();
    let root_path = config.root_path();

    for movie in classified.matches {
        println!("Starting copy for {}", movie.path().display());
        println!();

        let identity = movie.identity().expect("match identity should never be None");
        let title = &identity.title;

        let mut manager = Manager::new();

        let ext = movie.path().extension().and_then(|s| s.to_str()).unwrap_or("");
        let movie_path = make_movie_path(&title.primary_title, title.year, ext);

        manager.add_transfer(&movie.path(), root_path.join(&movie_path));

        let mut lib_movie = library::Movie::new(
            library::File::new(movie_path.clone(), movie.fingerprint.clone()),
            title.title_id,
            title.primary_title.clone(),
            title.original_title.clone().unwrap_or(title.primary_title.clone()),
            title.year,
        );

        for sub in movie.subtitles.iter() {
            let subtitle_path = make_subtitle_path(&movie_path, &sub);

            manager.add_transfer(&sub.path(), root_path.join(&subtitle_path));

            lib_movie.subtitles.push(library::Subtitle::new(
                library::File::new(subtitle_path, sub.fingerprint.clone()),
                sub.lang.clone(),
            ));

            // TODO: better duplicate subtitle selection
            lib_movie
                .subtitles
                .sort_unstable_by(|left, right| left.file.path.cmp(&right.file.path));

            lib_movie
                .subtitles
                .dedup_by(|left, right| left.file.path == right.file.path);
        }

        let mut last = Instant::now();
        loop {
            if QUIT.load(Ordering::Relaxed) {
                // received SIGINT or SIGTERM, remove incomplete transfer
                println!("\nReceived quit signal, cancelling current transfer.");
                manager.try_cancel();
                return Ok(());
            }
            match manager.step() {
                Ok(Some(transfer)) => {
                    if last.elapsed() > Duration::from_secs(1) {
                        print_transfer(transfer);
                        last = Instant::now();
                    }
                }
                Ok(None) => break,
                Err(err) => {
                    // IO error occured
                    println!("IO error {}, cancelling current transfer.", err);
                    manager.try_cancel();
                    return Err(err);
                }
            }
        }

        finished += 1;

        println!("Transfer status");
        println!("===============");

        for transfer in manager.transfers() {
            print_transfer(transfer);
        }

        println!("{}/{} files transfered", finished, len);
        println!("");

        library.save_movie(&lib_movie)?;
    }

    Ok(())
}

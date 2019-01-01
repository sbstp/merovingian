use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use signal_hook::flag as signal;
use signal_hook::{SIGINT, SIGTERM};

use crate::mero::{fingerprint, library, utils::clean_path, Index, Library, Manager, Result, SubtitleFile, Transfer};
use crate::storage::{Config, Report};

use super::view::Classified;

fn make_movie_path(primary_title: &str, year: u16, ext: &str) -> PathBuf {
    let mut path = PathBuf::new();
    let cleaned_name = clean_path(&format!("{} ({})", primary_title, year));
    let dotted_name = cleaned_name.replace(" ", ".");
    path.push(&dotted_name);
    path.push(format!("{}.{}", dotted_name, ext));
    path
}

fn make_subtitle_path(movie_path: &Path, subtitle: &SubtitleFile) -> PathBuf {
    let ext = format!("{}.{}", subtitle.lang, &subtitle.ext);
    movie_path.clone().with_extension(&ext)
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

pub fn cmd_apply(config: Config, path: impl AsRef<Path>, index: &Index, library: &mut Library) -> Result {
    signal::register(SIGINT, QUIT.clone()).expect("unable to setup SIGINT hook");
    signal::register(SIGTERM, QUIT.clone()).expect("unable to setup SIGTERM hook");

    let path = path.as_ref();

    let report = Report::load(path)?;
    let classified = Classified::classify(&library, report.movies);

    let mut finished = 0;
    let len = classified.matches.len();

    for movie in classified.matches {
        println!("Starting copy for {}", movie.path.display());
        println!();

        let title_id_scored = movie.title_id_scored.expect("score should not be None in apply");
        let title = index.get_title(title_id_scored.value);

        let mut manager = Manager::new();

        let ext = movie.path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let movie_path = config
            .root_path()
            .join(make_movie_path(&title.primary_title, title.year, ext));

        manager.add_transfer(&movie.path, &movie_path);

        let mut lib_subtitles = vec![];

        for sub in movie.subtitles {
            let subtitle_path = make_subtitle_path(&movie_path, &sub);
            lib_subtitles.push(library::Subtitle {
                path: subtitle_path.to_owned(),
                fingerprint: fingerprint::file(&sub.path)?,
            });
            manager.add_transfer(sub.path, subtitle_path);
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

        library.add_movie(title, movie_path, movie.fingerprint, lib_subtitles);
        library.commit()?;
    }

    Ok(())
}

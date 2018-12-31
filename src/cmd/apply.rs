use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::mero::{fingerprint, library, utils::clean_path, Index, Library, Manager, Result, Transfer};
use crate::storage::{Config, Scan, Subtitle};

fn make_movie_path(primary_title: &str, year: u16, ext: &str) -> PathBuf {
    let mut path = PathBuf::new();
    let cleaned_name = clean_path(&format!("{} ({})", primary_title, year));
    let dotted_name = cleaned_name.replace(" ", ".");
    path.push(&dotted_name);
    path.push(format!("{}.{}", dotted_name, ext));
    path
}

fn make_subtitle_path(movie_path: &Path, subtitle: &Subtitle) -> PathBuf {
    let ext = format!("{}.{}", subtitle.lang, &subtitle.ext);
    movie_path.clone().with_extension(&ext)
}

fn print_transfer(transfer: &Transfer) {
    println!("Source: {}", transfer.src.display());
    println!("Destination: {}", transfer.dst.display());
    println!("Status: {}", transfer.status());
    println!();
}

pub fn cmd_apply(config: Config, path: impl AsRef<Path>, index: &Index, library: &mut Library) -> Result {
    let path = path.as_ref();

    let scan = Scan::load(path)?;

    for mat in scan.matches {
        if library.has_fingerprint(&mat.fingerprint) {
            println!("Movie has already been added to the library. Skipping.");
            println!("If you have deleted the movie from your disk and are trying to re-import it,");
            println!("make sure to run the sync command to reflect the changes you made on disk into");
            println!("the library.");
            continue;
        }

        let title = &index.entries[&mat.title_id];

        let mut manager = Manager::new();

        let ext = mat.path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let movie_path = config
            .root_path()
            .join(make_movie_path(&title.primary_title, title.year, ext));

        manager.add_transfer(&mat.path, &movie_path);

        let mut lib_subtitles = vec![];

        for sub in mat.subtitles {
            let subtitle_path = make_subtitle_path(&movie_path, &sub);
            lib_subtitles.push(library::Subtitle {
                path: subtitle_path.to_owned(),
                fingerprint: fingerprint::file(&sub.path)?,
            });
            manager.add_transfer(sub.path, subtitle_path);
        }

        let mut x = Instant::now();
        while manager.has_work() {
            manager.tick();
            if x.elapsed() > Duration::from_secs(1) && manager.has_work() {
                print_transfer(manager.current());
                x = Instant::now();
            }
        }

        println!("Transfer status");
        println!("===============");

        for transfer in manager.transfers() {
            print_transfer(transfer);
        }

        println!("===============");

        library.add_movie(title, movie_path, mat.fingerprint, lib_subtitles);
        library.commit()?;
    }

    Ok(())
}

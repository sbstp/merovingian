use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use tera::Tera;

use crate::cmd::scan::Report;
use crate::index::{Title, TitleId};
use crate::mero::{Library, Result};
use crate::scan::{MovieFile, PathSize};
use crate::utils::NonNan;

#[derive(Serialize)]
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
            if let Some(identity) = movie.identity.as_ref() {
                if library.has_fingerprint(&movie.fingerprint) {
                    ignored.push(movie);
                } else {
                    if library.has_title(identity.value.title.title_id) {
                        duplicates.push(movie);
                    } else {
                        movies_by_title
                            .entry(identity.value.title.title_id)
                            .or_insert(Vec::new())
                            .push(movie);
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

        matches.sort_by_key(|m| m.identity.as_ref().expect("identity should not be None in sort").score);
        duplicates.sort_by_key(|m| m.identity.as_ref().expect("identity should not be None in sort").score);

        Classified {
            ignored,
            unmatched,
            duplicates,
            matches,
            conflicts,
        }
    }
}

fn fmt_filename(path: impl AsRef<Path>) -> String {
    Path::new(path.as_ref().file_name().expect("empty filename"))
        .display()
        .to_string()
}

fn fmt_score(score: NonNan) -> String {
    format!("{:0.3}", score)
}

fn fmt_size(size: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;
    const STEPS: [u64; 4] = [TIB, GIB, MIB, KIB];
    const LABELS: [&str; 4] = ["TiB", "GiB", "MiB", "KiB"];

    for (&step, label) in STEPS.iter().zip(LABELS.iter()) {
        if size >= step {
            return format!("{:0.2} {}", size as f64 / step as f64, label);
        }
    }

    format!("{} bytes", size)
}

#[derive(Serialize)]
struct TitleDto {
    primary_title: String,
    original_title: Option<String>,
    year: u16,
    url: String,
}

impl From<&Title> for TitleDto {
    fn from(title: &Title) -> TitleDto {
        TitleDto {
            primary_title: title.primary_title.clone(),
            original_title: title.original_title.clone(),
            year: title.year,
            url: format!("https://imdb.com/title/{}/", title.title_id.full()),
        }
    }
}

#[derive(Serialize)]
struct PathDto {
    filename: String,
    path: String,
    size: String,
}

impl From<&PathSize> for PathDto {
    fn from(pathsize: &PathSize) -> PathDto {
        PathDto {
            filename: fmt_filename(&pathsize.path),
            path: pathsize.path.display().to_string(),
            size: fmt_size(pathsize.size),
        }
    }
}

#[derive(Serialize)]
struct MatchInfoDto {
    path: PathDto,
    score: String,
}

impl From<&MovieFile> for MatchInfoDto {
    fn from(file: &MovieFile) -> MatchInfoDto {
        let scored = file.identity.as_ref().expect("identity is none");

        MatchInfoDto {
            path: From::from(file.pathsize()),
            score: fmt_score(scored.score),
        }
    }
}

#[derive(Serialize)]
struct MatchDto {
    title: TitleDto,
    info: MatchInfoDto,
}

impl From<&MovieFile> for MatchDto {
    fn from(file: &MovieFile) -> MatchDto {
        let scored = file.identity.as_ref().expect("identity is none");

        MatchDto {
            title: From::from(&scored.value.title),
            info: file.into(),
        }
    }
}

#[derive(Serialize)]
struct ConflictDto {
    title: TitleDto,
    paths: Vec<MatchInfoDto>,
}

impl From<&[MovieFile]> for ConflictDto {
    fn from(conflicts: &[MovieFile]) -> ConflictDto {
        let identity = conflicts[0].identity.as_ref().expect("identity is none");
        ConflictDto {
            title: From::from(&identity.value.title),
            paths: conflicts.iter().map(From::from).collect(),
        }
    }
}

#[derive(Serialize)]
struct DisplayDto {
    matches: Vec<MatchDto>,
    conflicts: Vec<ConflictDto>,
    duplicates: Vec<MatchDto>,
    unmatched: Vec<PathDto>,
    ignored: Vec<PathDto>,
}

impl From<&Classified> for DisplayDto {
    fn from(classified: &Classified) -> DisplayDto {
        DisplayDto {
            matches: classified.matches.iter().map(From::from).collect(),
            conflicts: classified.conflicts.values().map(|cs| From::from(&cs[..])).collect(),
            duplicates: classified.duplicates.iter().map(From::from).collect(),
            unmatched: classified.unmatched.iter().map(|file| file.pathsize().into()).collect(),
            ignored: classified.ignored.iter().map(|file| file.pathsize().into()).collect(),
        }
    }
}

fn print_text_report(classified: &Classified) {
    println!("Ignored (files that were already imported)");
    println!("=======");
    for movie in &classified.ignored {
        println!("{}", movie.path().display());
    }
    println!();

    println!("Unmatched (files that could not be matched with a movie)");
    println!("=========");
    for movie in &classified.unmatched {
        println!("{}", movie.path().display());
    }
    println!();

    println!("Duplicates (different copy of a movie already in the library)");
    println!("==========");
    for movie in &classified.duplicates {
        println!("{}", movie.path().display());
    }
    println!();

    println!("Conflicts (different copies of the same movie, not yet in the library)");
    println!("=========");
    for (_, movies) in classified.conflicts.iter() {
        let title = &movies
            .first()
            .and_then(|m| m.identity.as_ref())
            .expect("identity should not be None in conflicts")
            .value
            .title;
        println!("Title: {}", title.primary_title);
        println!("Year: {}", title.year);
        println!("URL: https://imdb.com/title/{}/", title.title_id.full());
        for movie in movies {
            println!("Path: {}", movie.path().display());
        }
        println!();
    }

    println!("Matches (movies to be imported)");
    println!("=======");
    for movie in &classified.matches {
        let identity = movie.identity.as_ref().expect("identity should not be None in print");
        let title = &identity.value.title;
        println!("Path: {}", movie.path().display());
        println!("Name: {}", movie.path().file_name().and_then(|s| s.to_str()).unwrap());
        println!("Title: {}", title.primary_title);
        println!("Year: {}", title.year);
        println!("URL: https://imdb.com/title/{}/", title.title_id.full());
        println!("Score: {:0.3}", identity.score);
        println!();
    }
}

pub fn cmd_view(path: impl AsRef<Path>, library: &Library, no_html: bool) -> Result {
    let path = path.as_ref();

    let report = Report::load(path)?;
    let classified = Classified::classify(library, report.movies);
    let display = DisplayDto::from(&classified);

    if !no_html {
        let mut tera = Tera::default();
        tera.add_raw_template("view_macros.html", include_str!("html/view_macros.html"))
            .expect("unable to compile view_macros.html");
        tera.add_raw_template("view.html", include_str!("html/view.html"))
            .expect("unable to compile view.html");

        let html_path = env::temp_dir().join("mero-view-report.html");
        let mut file = File::create(&html_path)?;
        write!(
            file,
            "{}",
            tera.render("view.html", &display).expect("error rendering report")
        )?;
        file.flush()?;
        if open::that(&html_path).is_err() {
            print_text_report(&classified);
        }
    } else {
        print_text_report(&classified);
    }

    Ok(())
}

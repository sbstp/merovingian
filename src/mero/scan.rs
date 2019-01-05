use std::fs;
use std::io::BufReader;
use std::path::PathBuf;

use chardet;
use encoding;
use hashbrown::HashSet;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use subparse::{self, SubtitleFormat};
use whatlang;

use super::tmdb::{self, TMDB};
use super::{fingerprint, File, Fingerprint, Index, Result, SafeBuffer, Scored, Title};

lazy_static! {
    static ref VIDEO_EXT: Vec<&'static str> =
        vec!["mkv", "mp4", "avi", "m4v", "webm", "flv", "vob", "mov", "wmv", "ogv", "ogg"];
    static ref SUBTITLE_EXT: Vec<&'static str> = vec!["srt", "sub", "ssa", "ass"];
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MovieIdentity {
    pub title: Title,
    pub tmdb_title: tmdb::Title,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MovieFile {
    pub path: PathBuf,
    pub identity: Option<Scored<MovieIdentity>>,
    pub fingerprint: Fingerprint,
    pub subtitles: Vec<SubtitleFile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SubtitleFile {
    pub path: PathBuf,
    pub lang: String,
    pub ext: String,
    pub fingerprint: Fingerprint,
}

fn is_video(file: &File) -> bool {
    file.is_file() && VIDEO_EXT.contains(&file.ext().to_lowercase().as_str())
}

fn is_subtitle(file: &File) -> bool {
    file.is_file() && SUBTITLE_EXT.contains(&file.ext().to_lowercase().as_str())
}

fn token_splitter(c: char) -> bool {
    match c {
        c if c.is_whitespace() => true,
        '_' | '-' | '.' | '(' | ')' | '[' | ']' | ':' => true,
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

fn parse_file_name(stem: &str) -> Option<(String, i32)> {
    let stem = stem.to_lowercase();
    let mut tokens = Vec::new();
    text_to_tokens(&stem, &mut tokens);

    let (year_pos, year_token) = tokens.iter().enumerate().rev().find(|(_, t)| is_year(t))?;

    let title = tokens[..year_pos].join(" ");
    let year = year_token.parse().expect("invalid year");

    Some((title, year))
}

pub struct Scanner {
    buff: SafeBuffer,
    tmdb: TMDB,
}

impl Scanner {
    pub fn new(tmdb: TMDB) -> Scanner {
        Scanner {
            buff: SafeBuffer::new(),
            tmdb: tmdb,
        }
    }

    fn analyze_subtitle(&mut self, file: &File) -> Option<SubtitleFile> {
        let mut fd = BufReader::new(fs::File::open(file.path()).ok()?);

        // Only read the first 512 bytes to scan for the format.
        // VoSub being images, the files are really large. Since
        // these files are ignored anyway, we don't need to read them fully.
        self.buff.clear();
        self.buff.read_exact(&mut fd, 512).ok()?;

        let format = subparse::get_subtitle_format(&format!(".{}", file.ext().to_lowercase()), &self.buff)?;
        if format == SubtitleFormat::VobSubSub || format == SubtitleFormat::VobSubIdx {
            return None;
        }

        // Once we know this subtitle file is actually something we care about,
        // we can read it fully into a re-usable buffer. The bytes are appended
        // to the buffer, so there's no need to seek from the start and re-read
        // the bytes.
        self.buff.read_to_end(&mut fd).ok()?;

        // detect the encoding
        let (charset, _, _) = chardet::detect(&self.buff);
        let encoding = encoding::label::encoding_from_whatwg_label(chardet::charset2encoding(&charset))?;

        // parse the subtitle file
        let sub = subparse::parse_bytes(format, &self.buff, encoding, 30.0).ok()?;

        // join all the text segments into a string
        let mut text = String::new();
        for entry in sub.get_subtitle_entries().ok()? {
            if let Some(line) = entry.line {
                text.push_str(&line);
            }
        }

        if text.is_empty() {
            return None;
        }

        // detect language
        let lang = whatlang::detect(&text)?.lang();

        let fp = fingerprint::bytes(&self.buff);

        Some(SubtitleFile {
            path: file.path().to_owned(),
            ext: format.get_name()[1..4].to_owned(),
            lang: lang.code().to_owned(),
            fingerprint: fp,
        })
    }

    /// Scan for subtitles around a movie file.
    fn scan_subtitles(&mut self, movie: &File) -> Vec<SubtitleFile> {
        let mut subs = vec![];
        for file in movie.siblings() {
            if is_subtitle(&file) && file.name().starts_with(movie.stem()) {
                println!("Analyzing subtitle {}", file.path().display());
                if let Some(sub) = self.analyze_subtitle(&file) {
                    subs.push(sub);
                }
            }
        }
        subs
    }

    /// Scan for files that look like movies.
    pub fn scan_movies<'i>(&mut self, root: &File, index: &Index) -> Result<Vec<MovieFile>> {
        let mut ignored: HashSet<File> = HashSet::new();
        let mut results: Vec<(File, MovieFile)> = Vec::new();

        for child in root.descendants() {
            if is_video(&child) {
                if let Some((title, year)) = parse_file_name(child.stem()) {
                    // Once we find a movie we try to look for peers that are small.
                    // Usually featurettes, samples and extras and mark them as ignored.
                    if let Some(parent) = child.parent() {
                        if parent != *root {
                            let size = child.metadata().len() as f64;

                            for peer in parent.descendants() {
                                if peer.metadata().len() as f64 / size <= 0.40 {
                                    ignored.insert(peer);
                                }
                            }
                        }
                    }

                    let mut identity = None;

                    if let Some(scored) = index.find(&title, Some(year)) {
                        let title = scored.value;
                        println!("Looking up info on themoviedb.org for {}", child.path().display());
                        if let Some(tmdb_title) = self.tmdb.find(title.title_id)? {
                            identity = Some(Scored::new(
                                scored.score,
                                MovieIdentity {
                                    title: title.clone(),
                                    tmdb_title: tmdb_title,
                                },
                            ));
                        }
                    }

                    results.push((
                        child.clone(),
                        MovieFile {
                            path: child.path().to_owned(),
                            identity: identity,
                            // We use a null fingerprint here because we want to avoid fingerprinting
                            // files that will be removed as ignored.
                            fingerprint: Fingerprint::null(),
                            subtitles: vec![],
                        },
                    ));
                }
            }
        }

        // Remove every file that was flagged as ignored from the list.
        results.retain(|(file, _)| !ignored.contains(&file));

        // Fingerprint and scan for subtitles for each remaining movie file.
        for (file, movie) in results.iter_mut() {
            println!("Scanning subtitles for {}", movie.path.display());
            movie.fingerprint = fingerprint::file(&movie.path)?;
            movie.subtitles = self.scan_subtitles(&file);
        }

        Ok(results.into_iter().map(|(_, movie)| movie).collect())
    }
}

#[test]
fn test_parse_file_name_simple() {
    assert_eq!(
        parse_file_name("American Psycho 1999"),
        Some(("american psycho".to_string(), 1999))
    );

    assert_eq!(
        parse_file_name("American_Psycho_(1999)"),
        Some(("american psycho".to_string(), 1999))
    );

    assert_eq!(
        parse_file_name("American.Psycho.[1999]"),
        Some(("american psycho".to_string(), 1999))
    );
}

#[test]
fn test_parse_file_name_with_year() {
    assert_eq!(
        parse_file_name("2001: A Space Odyssey (1968)"),
        Some(("2001 a space odyssey".to_string(), 1968))
    );

    assert_eq!(parse_file_name("1981.(2009)"), Some(("1981".to_string(), 2009)));
}

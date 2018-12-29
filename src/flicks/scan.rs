use std::fs;
use std::io::Read;

use chardet;
use encoding;
use hashbrown::HashSet;
use lazy_static::lazy_static;
use subparse::{self, SubtitleFormat};
use whatlang::{self, Lang};

use super::index::{Entry, Index};
use super::vfs::File;

lazy_static! {
    static ref VIDEO_EXT: Vec<&'static str> =
        vec!["mkv", "mp4", "avi", "m4v", "webm", "flv", "vob", "mov", "wmv", "ogv", "ogg"];
    static ref SUBTITLE_EXT: Vec<&'static str> = vec!["srt", "sub", "ssa", "ass"];
}

#[derive(Debug)]
pub struct ScanResult<'i> {
    pub movie: File,
    pub year: i32,
    pub title: String,
    pub entry: Option<&'i Entry>,
    pub subtitles: Vec<SubtitleFile>,
}

#[derive(Debug)]
pub struct SubtitleFile {
    pub file: File,
    pub lang: Lang,
    pub format: SubtitleFormat,
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

fn analyze_subtitle(file: &File) -> Option<SubtitleFile> {
    let mut fd = fs::File::open(file.path()).ok()?;
    let mut contents = Vec::new();
    fd.read_to_end(&mut contents).ok()?;

    // detect the encoding
    let (charset, _, _) = chardet::detect(&contents);
    let encoding = encoding::label::encoding_from_whatwg_label(chardet::charset2encoding(&charset))?;

    // parse the subtitle file
    let format = subparse::get_subtitle_format(&format!(".{}", file.ext().to_lowercase()), &contents)?;
    let sub = subparse::parse_bytes(format, &contents, encoding, 30.0).ok()?;

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

    Some(SubtitleFile {
        file: file.clone(),
        format: format,
        lang: lang,
    })
}

fn scan_subtitles(movie: &File) -> Vec<SubtitleFile> {
    let mut subs = vec![];
    for file in movie.siblings() {
        if is_subtitle(&file) && file.name().starts_with(movie.stem()) {
            if let Some(sub) = analyze_subtitle(&file) {
                subs.push(sub);
            }
        }
    }
    subs
}

pub fn scan<'i>(root: &File, index: &'i Index) -> Vec<ScanResult<'i>> {
    let mut ignored: HashSet<File> = HashSet::new();
    let mut results: Vec<ScanResult> = Vec::new();

    for child in root.descendants() {
        if is_video(&child) {
            if let Some((title, year)) = parse_title(child.stem()) {
                // once we find a movie we try to look for peers that are small
                // (usually featurettes, samples and extras) and mark them as ignored
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

                results.push(ScanResult {
                    entry: index.best_match(&title, Some(year)),
                    title: title,
                    year: year,
                    movie: child,
                    subtitles: vec![],
                });
            }
        }
    }

    results.retain(|sr| !ignored.contains(&sr.movie));

    for sr in results.iter_mut() {
        sr.subtitles.extend(scan_subtitles(&sr.movie));
    }

    results
}

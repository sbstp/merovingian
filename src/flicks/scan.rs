use std::fs;
use std::io::{BufReader, Read};

use chardet;
use encoding;
use hashbrown::HashSet;
use lazy_static::lazy_static;
use log::debug;
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

fn parse_title(stem: &str) -> Option<(String, i32)> {
    let stem = stem.to_lowercase();
    let mut tokens = Vec::new();
    text_to_tokens(&stem, &mut tokens);

    let (year_pos, year_token) = tokens.iter().enumerate().rev().find(|(_, t)| is_year(t))?;

    let title = tokens[..year_pos].join(" ");
    let year = year_token.parse().expect("invalid year");

    Some((title, year))
}

pub struct Scanner {
    buff: Vec<u8>,
}

impl Scanner {
    pub fn new() -> Scanner {
        Scanner {
            buff: Vec::with_capacity(500 * 1024), // 500 KiB should be enough for most files
        }
    }

    fn analyze_subtitle(&mut self, file: &File) -> Option<SubtitleFile> {
        debug!("analyzing subtitle {}", file.path().display());

        self.buff.clear();
        let mut fd = BufReader::new(fs::File::open(file.path()).ok()?);

        // Only read the first 512 bytes to scan for the format.
        // VoSub being images, the files are really large. Since
        // these files are ignored anyway, we don't need to read them fully.
        //
        // Capacity of the buffer is at least 500 KiB, so set_len is safe.
        // We set the len to 512 so that `read_exact` will read 512 bytes.
        unsafe {
            self.buff.set_len(512);
        }
        fd.read_exact(&mut self.buff[..512]).ok()?;
        let format = subparse::get_subtitle_format(&format!(".{}", file.ext().to_lowercase()), &self.buff)?;
        if format == SubtitleFormat::VobSubSub || format == SubtitleFormat::VobSubIdx {
            return None;
        }

        // Once we know this subtitle file is actually something we care about,
        // we can read it fully into a re-usable buffer. The bytes are appended
        // to the buffer, so there's no need to seek from the start and re-read
        // the bytes.
        fd.read_to_end(&mut self.buff).ok()?;

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

        Some(SubtitleFile {
            file: file.clone(),
            format: format,
            lang: lang,
        })
    }

    fn scan_subtitles(&mut self, movie: &File) -> Vec<SubtitleFile> {
        debug!("scanning subtitles for {}", movie.path().display());

        let mut subs = vec![];
        for file in movie.siblings() {
            if is_subtitle(&file) && file.name().starts_with(movie.stem()) {
                if let Some(sub) = self.analyze_subtitle(&file) {
                    subs.push(sub);
                }
            }
        }
        subs
    }

    pub fn scan<'i>(mut self, root: &File, index: &'i Index) -> Vec<ScanResult<'i>> {
        let mut ignored: HashSet<File> = HashSet::new();
        let mut results: Vec<ScanResult> = Vec::new();

        for child in root.descendants() {
            debug!("child {}", child.path().display());
            if is_video(&child) {
                debug!("child is a video {}", child.path().display());
                if let Some((title, year)) = parse_title(child.stem()) {
                    debug!("child is movie {}", child.path().display());
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
            sr.subtitles.extend(self.scan_subtitles(&sr.movie));
        }

        results
    }
}

#[test]
fn test_parse_title_simple() {
    assert_eq!(
        parse_title("American Psycho 1999"),
        Some(("american psycho".to_string(), 1999))
    );

    assert_eq!(
        parse_title("American_Psycho_(1999)"),
        Some(("american psycho".to_string(), 1999))
    );

    assert_eq!(
        parse_title("American.Psycho.[1999]"),
        Some(("american psycho".to_string(), 1999))
    );
}

#[test]
fn test_parse_title_with_year() {
    assert_eq!(
        parse_title("2001: A Space Odyssey (1968)"),
        Some(("2001 a space odyssey".to_string(), 1968))
    );

    assert_eq!(parse_title("1981.(2009)"), Some(("1981".to_string(), 2009)));
}

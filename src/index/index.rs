use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use csv;
use libflate::gzip::Decoder;
use serde::{Deserialize, Serialize};

use super::counter::Counter;
use super::fixed_string::FixedString;
use crate::mero::error::Result;
use crate::utils::{self, NonNan};

const MIN_VOTES: u32 = 25;

#[derive(Debug, Deserialize, Serialize, Copy, Clone, Eq, PartialEq, Hash)]
pub struct TitleId(u32);

impl TitleId {
    pub fn new(id: u32) -> TitleId {
        TitleId(id)
    }

    pub fn full(&self) -> String {
        format!("tt{:07}", self.0)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Title {
    pub title_id: TitleId,
    pub primary_title: String,
    pub original_title: Option<String>,
    pub year: u16,
    pub runtime: u16,
    pub vote_count: u32,
}

fn open_csv(path: &Path) -> Result<csv::Reader<Decoder<File>>> {
    let file = File::open(path)?;
    Ok(csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .quoting(false)
        .from_reader(Decoder::new(file)?))
}

fn parse_none<T: FromStr>(record: &str) -> Option<T> {
    match record {
        "\\N" => None,
        s => s.parse().ok(),
    }
}

fn parse_id(record: &str) -> Result<TitleId> {
    Ok(TitleId(record[2..].parse()?))
}

fn is_valid_type(title_type: &str) -> bool {
    match title_type {
        "movie" | "tvMovie" | "video" | "short" => true,
        _ => false,
    }
}

fn build_titles_table(data_dir_path: &Path) -> Result<HashMap<TitleId, Title>> {
    let mut ratings_reader = open_csv(&data_dir_path.join("title.ratings.tsv.gz"))?;
    let mut titles_reader = open_csv(&data_dir_path.join("title.basics.tsv.gz"))?;

    let mut votes_table: HashMap<TitleId, u32> = HashMap::new();

    for record in ratings_reader.records() {
        let record = record?;
        let title_id = parse_id(&record[0])?;
        let vote_count: u32 = record[2].parse()?;
        if vote_count >= MIN_VOTES {
            votes_table.insert(title_id, vote_count);
        }
    }

    let mut titles_table: HashMap<TitleId, Title> = HashMap::new();

    for record in titles_reader.records() {
        let record = record?;
        let title_id = parse_id(&record[0])?;
        let title_type = &record[1];
        let primary_title = &record[2];
        let original_title = &record[3];
        let adult = &record[4];
        let start_year = parse_none(&record[5]);
        let runtime = parse_none(&record[5]);
        let vote_count = votes_table.get(&title_id);

        match (is_valid_type(title_type), adult, start_year, runtime, vote_count) {
            (true, "0", Some(start_year), Some(runtime), Some(vote_count)) => {
                let title = Title {
                    title_id: title_id,
                    primary_title: primary_title.into(),
                    original_title: if primary_title != original_title {
                        Some(original_title.into())
                    } else {
                        None
                    },
                    year: start_year,
                    runtime: runtime,
                    vote_count: *vote_count,
                };
                titles_table.insert(title_id, title);
            }
            _ => {}
        }
    }

    titles_table.shrink_to_fit();
    Ok(titles_table)
}

// Token splitter must be a superset of the filter_path function
fn token_splitter(c: char) -> bool {
    match c {
        c if c.is_whitespace() => true,
        c if c.is_ascii_control() => true,
        '/' | '<' | '>' | ':' | '"' | '\\' | '|' | '?' | '*' => true, // from filter_path
        '_' => true,
        '-' => true,
        '.' => true,
        ',' => true,
        '\'' => true,
        '(' => true,
        ')' => true,
        _ => false,
    }
}

fn is_ignored_token(token: &str) -> bool {
    match token {
        "a" | "an" | "the" | "of" | "in" | "on" | "to" | "t" | "s" => true,
        _ => false,
    }
}

fn text_to_tokens(text: &str, tokens: &mut Vec<FixedString>) {
    let text = text.to_lowercase();
    tokens.clear();
    for token in text.split(token_splitter) {
        if !token.is_empty() && !is_ignored_token(token) {
            tokens.push(FixedString::new(token));
        }
    }
    tokens.dedup();
}

fn build_reverse_lookup_table(titles: &HashMap<TitleId, Title>) -> HashMap<FixedString, HashSet<TitleId>> {
    let mut table = HashMap::new();
    let mut tokens = Vec::new();

    for title in titles.values() {
        let mut index_title = |text: &str| {
            text_to_tokens(&text, &mut tokens);
            for tag in tokens.drain(..) {
                table
                    .entry(tag)
                    .or_insert_with(|| HashSet::new())
                    .insert(title.title_id);
            }
        };

        index_title(&title.primary_title);
        if let Some(original_title) = &title.original_title {
            index_title(&original_title);
        }
    }

    table
}

fn most_common(counter: &Counter<TitleId>) -> Vec<TitleId> {
    if let Some(&max) = counter.values().max() {
        let max = max as i64 - 1;
        counter
            .iter()
            .filter(|(_, &count)| count as i64 >= max)
            .map(|(&key, _)| key)
            .collect()
    } else {
        vec![]
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Scored<T> {
    pub score: NonNan,
    pub value: T,
}

impl<T> Scored<T> {
    pub fn new(score: NonNan, value: T) -> Scored<T> {
        Scored { score: score, value }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Index {
    titles: HashMap<TitleId, Title>,
    reverse: HashMap<FixedString, HashSet<TitleId>>,
}

impl Index {
    pub fn create_index(data_dir: &Path) -> Result<Index> {
        let titles = build_titles_table(data_dir)?;
        let reverse = build_reverse_lookup_table(&titles);

        Ok(Index { titles, reverse })
    }

    pub fn load_index(path: impl AsRef<Path>) -> Result<Index> {
        let mut index: Index = utils::deserialize_bin_gz(path)?;

        index.titles.shrink_to_fit();
        index.reverse.shrink_to_fit();
        index.reverse.values_mut().for_each(|bucket| bucket.shrink_to_fit());

        Ok(index)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        utils::serialize_bin_gz(path, self)
    }

    pub fn get_title(&self, title_id: TitleId) -> &Title {
        &self.titles[&title_id]
    }

    pub fn find_all(&self, text: &str, year: Option<i32>) -> Vec<Scored<&Title>> {
        let text = text.to_lowercase();
        let mut tokens = vec![];

        text_to_tokens(&text, &mut tokens);

        let mut matches: Counter<TitleId> = Counter::new();
        for token in tokens.drain(..) {
            if !is_ignored_token(&token) {
                if let Some(title_ids) = self.reverse.get(&token) {
                    matches.extend(title_ids.iter().cloned());
                }
            }
        }

        let mut titles: Vec<&Title> = most_common(&matches)
            .into_iter()
            .map(|title_id| &self.titles[&title_id])
            .collect();

        if let Some(year) = year {
            titles.retain(|t| (t.year as i32 - year).abs() <= 1);
        }

        if let Some(max_votes) = titles.iter().map(|title| title.vote_count).max() {
            let scoring_func = |title: &Title| -> NonNan {
                let mut score = match &title.original_title {
                    None => strsim::normalized_levenshtein(&title.primary_title.to_lowercase(), &text),
                    Some(original_title) => f64::max(
                        strsim::normalized_levenshtein(&title.primary_title.to_lowercase(), &text),
                        strsim::normalized_levenshtein(&original_title.to_lowercase(), &text),
                    ),
                };

                if let Some(year) = year {
                    if title.year as i32 != year {
                        score *= 0.90;
                    }
                }

                let popularity = f64::log10(title.vote_count as f64) / f64::log10(max_votes as f64);
                score *= popularity;

                NonNan::new(score)
            };

            let mut scored: Vec<Scored<&Title>> = titles.iter().map(|&e| Scored::new(scoring_func(e), e)).collect();
            scored.sort_by_key(|s| std::cmp::Reverse(s.score));
            scored
        } else {
            vec![]
        }
    }

    pub fn find(&self, text: &str, year: Option<i32>) -> Option<Scored<&Title>> {
        self.find_all(text, year).into_iter().next()
    }
}

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::thread;
use std::time::Duration;

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::mero::{Result, TitleId};

const API_KEY: &'static str = "89049522cb87421d059ed3fd5bae460c";

#[derive(Debug, Deserialize, Serialize)]
pub struct MovieInfo {
    pub id: u32,
    pub title: String,
    pub original_title: String,
    pub original_language: String,
    pub overview: String,
    pub release_date: String,
    pub popularity: f64,
    pub vote_count: u32,
    pub vote_average: f64,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub adult: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct MovieResult {
    movie_results: Vec<MovieInfo>,
}

pub struct TMDB {
    client: Client,
}

impl TMDB {
    pub fn new() -> TMDB {
        TMDB { client: Client::new() }
    }

    pub fn find(&self, title_id: TitleId) -> Result<Option<MovieInfo>> {
        let url = format!(
            "https://api.themoviedb.org/3/find/{}?api_key={}&external_source=imdb_id",
            title_id.full(),
            API_KEY
        );
        let mut resp = self.client.get(&url).send()?;
        if resp.status().is_success() {
            let result: MovieResult = resp.json()?;
            thread::sleep(Duration::from_millis(250));
            Ok(result.movie_results.into_iter().next())
        } else {
            thread::sleep(Duration::from_millis(250));
            Ok(None)
        }
    }

    pub fn get_save_image(&self, path: &str, outpath: impl AsRef<Path>) -> Result<()> {
        let mut writer = BufWriter::new(File::create(outpath.as_ref())?);
        let url = format!("https://image.tmdb.org/t/p/original/{}", path);
        let mut resp = self.client.get(&url).send()?;
        if resp.status().is_success() {
            resp.copy_to(&mut writer)?;
        }
        Ok(())
    }
}

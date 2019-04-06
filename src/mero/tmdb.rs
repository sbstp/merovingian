use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::index::TitleId;
use crate::mero::Result;
use crate::utils;

const API_KEY: &'static str = "89049522cb87421d059ed3fd5bae460c";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Title {
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
struct FindResult {
    movie_results: Vec<Title>,
}

pub struct TMDB {
    cache: HashMap<TitleId, Title>,
    cache_path: PathBuf,
}

impl TMDB {
    pub fn new(cache_path: impl Into<PathBuf>) -> TMDB {
        let cache_path = cache_path.into();
        TMDB {
            cache: TMDB::open_cache(&cache_path).unwrap_or_else(|| HashMap::new()),
            cache_path: cache_path,
        }
    }

    fn open_cache(path: &Path) -> Option<HashMap<TitleId, Title>> {
        utils::deserialize_bin_gz(&path).ok()
    }

    fn save_cache(&self) -> Result<()> {
        utils::serialize_bin_gz(&self.cache_path, &self.cache)
    }

    pub fn find(&mut self, title_id: TitleId) -> Result<Option<Title>> {
        if let Some(info) = self.cache.get(&title_id) {
            return Ok(Some(info.clone()));
        }

        let url = format!(
            "https://api.themoviedb.org/3/find/{}?api_key={}&external_source=imdb_id",
            title_id.full(),
            API_KEY
        );

        let resp = attohttpc::get(&url).send()?;
        thread::sleep(Duration::from_millis(250));

        if resp.is_success() {
            let result: FindResult = resp.json()?;
            let info = result.movie_results.into_iter().next();
            if let Some(info) = &info {
                self.cache.insert(title_id, info.clone());
                self.save_cache()?;
            }
            Ok(info)
        } else {
            Ok(None)
        }
    }

    pub fn get_save_image(&self, path: &str, outpath: impl AsRef<Path>) -> Result<()> {
        let mut writer = BufWriter::new(File::create(outpath.as_ref())?);
        let url = format!("https://image.tmdb.org/t/p/original/{}", path);
        let resp = attohttpc::get(&url).send()?;
        if resp.is_success() {
            resp.write_to(&mut writer)?;
        }
        Ok(())
    }
}

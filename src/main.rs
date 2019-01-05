#![allow(dead_code)]

mod cmd;
mod mero;
mod storage;

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use crate::mero::{error::Result, index::Index, library::Library};
use crate::storage::Config;

const SRC_FILE_BASICS: &str = "title.basics.tsv.gz";
const SRC_FILE_RATINGS: &str = "title.ratings.tsv.gz";

macro_rules! flush {
    () => {
        let _ = std::io::Write::flush(&mut std::io::stdout());
    };
}

pub fn task<F, T>(task_description: impl AsRef<str>, func: F) -> T
where
    F: FnOnce() -> T,
{
    print!("{} ... ", task_description.as_ref());
    flush!();
    let result = func();
    println!("done.");
    flush!();
    result
}

fn download_file(client: &reqwest::Client, url: &str, dest: impl AsRef<Path>) -> Result<()> {
    let mut file = BufWriter::new(File::create(dest)?);
    let mut resp = client.get(url).send()?;
    resp.copy_to(&mut file)?;
    Ok(())
}

fn download_file_if_missing(client: &reqwest::Client, url: &str, dest: impl AsRef<Path>) -> Result<()> {
    if !dest.as_ref().exists() {
        task(format!("Downloading {}", url), || download_file(client, url, dest))?;
    }
    Ok(())
}

fn check_source_files(data_dir: &Path) -> Result<()> {
    let client = reqwest::Client::new();

    download_file_if_missing(
        &client,
        "https://datasets.imdbws.com/title.basics.tsv.gz",
        data_dir.join(SRC_FILE_BASICS),
    )?;

    download_file_if_missing(
        &client,
        "https://datasets.imdbws.com/title.ratings.tsv.gz",
        data_dir.join(SRC_FILE_RATINGS),
    )?;

    Ok(())
}

pub fn load_or_create_index(config: &Config) -> Result<Index> {
    let data_dir = config.meta_dir();
    let index_path = config.index_path();

    check_source_files(&data_dir)?;

    task("Loading index", || match Index::load_index(&index_path) {
        Ok(index) => Ok(index),
        Err(_) => task("Generating index", || {
            let index = Index::create_index(&data_dir)?;
            index.save(&index_path)?;
            Ok(index)
        }),
    })
}

#[derive(StructOpt)]
#[structopt(name = "mero")]
enum App {
    #[structopt(name = "import", about = "Import movies matched by the given scan report")]
    Import {
        #[structopt(parse(from_os_str))]
        report: PathBuf,
    },
    #[structopt(name = "images", about = "Download images for movies in the database")]
    Images,
    #[structopt(name = "init", about = "Initialize merovingian with the given library path")]
    Init {
        #[structopt(parse(from_os_str))]
        directory: PathBuf,
    },
    #[structopt(name = "query", about = "")]
    Query {
        #[structopt(long = "title", help = "Title contains")]
        title: Option<String>,
        #[structopt(long = "year", help = "Exact year")]
        year: Option<u16>,
        #[structopt(long = "year-gte", help = "Year greater than or equal to")]
        year_gte: Option<u16>,
        #[structopt(long = "year-lte", help = "Year less than or equal to")]
        year_lte: Option<u16>,
    },
    #[structopt(name = "rehash", about = "Update fingerprints of movies and subtitles")]
    Rehash,
    #[structopt(name = "scan", about = "Scan a directory for movies")]
    Scan {
        #[structopt(parse(from_os_str))]
        directory: PathBuf,
        #[structopt(short = "o", help = "Output path for the scan report", parse(from_os_str))]
        out: Option<PathBuf>,
    },
    #[structopt(name = "stats", about = "View stats about the library")]
    Stats,
    #[structopt(name = "sync", about = "Synchronize changes made on disk to the library")]
    Sync,
    #[structopt(name = "view", about = "View a scan report file")]
    View {
        #[structopt(parse(from_os_str))]
        report: PathBuf,
    },
}

fn with_config<F>(func: F) -> Result
where
    F: FnOnce(Config) -> Result,
{
    match Config::open()? {
        Some(config) => func(config)?,
        None => println!("Initialize the config with the init command."),
    }
    Ok(())
}

fn open_all<F>(func: F) -> Result
where
    F: FnOnce(Config, Index, Library) -> Result,
{
    match Config::open()? {
        Some(config) => {
            let index = load_or_create_index(&config)?;
            let library = Library::open(&config.library_path())?;
            func(config, index, library)
        }?,
        None => println!("Initialize the config with the init command."),
    }
    Ok(())
}

fn open_library<F>(func: F) -> Result
where
    F: FnOnce(Config, Library) -> Result,
{
    match Config::open()? {
        Some(config) => {
            let library = Library::open(&config.library_path())?;
            func(config, library)
        }?,
        None => println!("Initialize the config with the init command."),
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = App::from_args();

    use crate::cmd::*;

    match args {
        App::Import { report } => {
            open_library(|config, mut library| cmd_import(config, report, &mut library))?;
        }
        App::Images => {
            open_library(|config, mut library| cmd_images(config, &mut library))?;
        }
        App::Init { directory } => {
            cmd_init(directory)?;
        }
        App::Query {
            title,
            year,
            year_gte,
            year_lte,
        } => {
            open_library(|_, library| cmd_query(&library, title, year, year_gte, year_lte))?;
        }
        App::Rehash => {
            open_library(|config, mut library| cmd_rehash(config, &mut library))?;
        }
        App::Scan { directory, out } => {
            open_all(|config, index, _| cmd_scan(&directory, out, config, &index))?;
        }
        App::Stats => {
            open_library(|_, library| cmd_stats(&library))?;
        }
        App::Sync => {
            open_library(|config, mut library| cmd_sync(config, &mut library))?;
        }
        App::View { report } => {
            open_library(|_, library| cmd_view(&report, &library))?;
        }
    }

    Ok(())
}

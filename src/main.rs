#![allow(dead_code)]

mod cmd;
mod config;
mod local_storage;
mod mero;

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use lynx::Request;
use structopt::StructOpt;

use crate::config::Config;
use crate::local_storage::LocalStorage;
use crate::mero::{error::Result, index::Index, library::Library};

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

fn download_file(url: &str, dest: impl AsRef<Path>) -> Result<()> {
    let (status, _, resp) = Request::get(url).send()?;
    if status.is_success() {
        let file = BufWriter::new(File::create(dest)?);
        resp.write_to(file)?;
        Ok(())
    } else {
        eprintln!("Error fetching '{}' : code {}", url, status.as_u16());
        panic!();
    }
}

fn download_file_if_missing(url: &str, dest: impl AsRef<Path>) -> Result<()> {
    if !dest.as_ref().exists() {
        task(format!("Downloading {}", url), || download_file(url, dest))?;
    }
    Ok(())
}

fn check_source_files(data_dir: &Path) -> Result<()> {
    download_file_if_missing(
        "https://datasets.imdbws.com/title.basics.tsv.gz",
        data_dir.join(SRC_FILE_BASICS),
    )?;

    download_file_if_missing(
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
enum CmdIgnore {
    #[structopt(name = "add", about = "Add an ignored file or directory")]
    Add {
        #[structopt(parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    #[structopt(name = "remove", about = "Remove an ignored file or directory")]
    Remove {
        #[structopt(parse(from_os_str))]
        paths: Vec<PathBuf>,
    },
    #[structopt(name = "list", about = "List ignored files and directories")]
    List,
}

#[derive(StructOpt)]
#[structopt(name = "mero")]
enum App {
    #[structopt(name = "ignore", about = "Managed ignored files")]
    Ignore(CmdIgnore),
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
    #[structopt(name = "query", about = "Query the library for movies")]
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
        #[structopt(short = "n", long = "no-html", help = "Do not show report in browser")]
        no_html: bool,
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
        App::Ignore(action) => {
            with_config(|config| {
                let local_storage = LocalStorage::open(config.local_storage_path())?;
                match action {
                    CmdIgnore::Add { paths } => {
                        cmd_ignore_add(&config, local_storage, paths)?;
                    }
                    CmdIgnore::Remove { paths } => {
                        cmd_ignore_remove(&config, local_storage, paths)?;
                    }
                    CmdIgnore::List => {
                        cmd_ignore_list(local_storage);
                    }
                }

                Ok(())
            })?;
        }
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
            open_all(|config, index, _| {
                let local_storage = LocalStorage::open(config.local_storage_path())?;
                cmd_scan(&directory, out, config, &index, &local_storage)
            })?;
        }
        App::Stats => {
            open_library(|_, library| cmd_stats(&library))?;
        }
        App::Sync => {
            open_library(|config, mut library| cmd_sync(config, &mut library))?;
        }
        App::View { report, no_html } => {
            open_library(|_, library| cmd_view(&report, &library, no_html))?;
        }
    }

    Ok(())
}

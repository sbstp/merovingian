#![allow(dead_code)]

mod cmd;
mod mero;
mod storage;

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

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

fn download_file(client: &reqwest::Client, url: &str, dest: impl AsRef<Path>) -> Result<()> {
    let mut file = BufWriter::new(File::create(dest)?);
    let mut resp = client.get(url).send()?;
    resp.copy_to(&mut file)?;
    Ok(())
}

fn download_file_if_missing(client: &reqwest::Client, url: &str, dest: impl AsRef<Path>) -> Result<()> {
    if !dest.as_ref().exists() {
        print!("Downloading {} ...", url);

        flush!();
        download_file(client, url, dest)?;

        println!("done.");
        flush!();
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

    Ok(match Index::load_index(&index_path) {
        Ok(index) => index,
        Err(_) => {
            print!("Generating index... ");
            flush!();

            let index = Index::create_index(&data_dir)?;
            index.save(&index_path)?;

            println!("done.");
            flush!();

            index
        }
    })
}

#[derive(StructOpt)]
#[structopt(name = "flicks")]
enum App {
    #[structopt(name = "apply", about = "Apply a scan result file")]
    Apply { scan: String },
    #[structopt(name = "init", about = "Initialize merovingian with the given library path")]
    Init { path: String },
    #[structopt(name = "scan", about = "Scan a directory for movies")]
    Scan { directory: String },
    #[structopt(name = "sync", about = "Synchronize changes made on disk to the library")]
    Sync,
    #[structopt(name = "view", about = "View a scan result file")]
    View { scan: String },
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

fn main() -> Result<()> {
    let args = App::from_args();

    match args {
        App::Apply { scan } => {
            with_config(|config| {
                let index = load_or_create_index(&config)?;
                let mut library = Library::open(&config.library_path())?;
                cmd::apply::cmd_apply(config, &scan, &index, &mut library)?;
                Ok(())
            })?;
        }
        App::Init { path } => {
            cmd::init::cmd_init(&path)?;
        }
        App::Scan { directory } => {
            with_config(|config| {
                let index = load_or_create_index(&config)?;
                let library = Library::open(&config.library_path())?;
                cmd::scan::cmd_scan(&directory, &index, &library)?;
                Ok(())
            })?;
        }
        App::View { scan } => {
            with_config(|config| {
                let index = load_or_create_index(&config)?;
                cmd::view::cmd_view(&scan, &index)?;
                Ok(())
            })?;
        }
        App::Sync => {
            with_config(|config| {
                let mut library = Library::open(&config.library_path())?;
                cmd::sync::cmd_sync(config, &mut library)?;
                Ok(())
            })?;
        }
    }

    Ok(())
}

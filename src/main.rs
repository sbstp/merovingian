#![allow(dead_code)]

mod cmd;
mod input;
mod mero;
mod storage;

use std::path::Path;

use structopt::StructOpt;

use crate::input::Input;
use crate::mero::{error::Result, index::Index, library::Library};

fn open_library(input: &Input) -> Library {
    match Library::open(".index/library.json") {
        Ok(lib) => lib,
        Err(_) => {
            let root = loop {
                let root = input.prompt("Where would you like to store your movie library?");
                match Path::new(&root).canonicalize() {
                    Ok(path) => {
                        if path.is_dir() {
                            break path;
                        } else {
                            println!("Not a directory");
                        }
                    }
                    Err(err) => println!("{}", err),
                }
            };
            Library::create(".index/library.json", root)
        }
    }
}

#[derive(StructOpt)]
#[structopt(name = "flicks")]
enum App {
    #[structopt(name = "apply", about = "Apply a scan result file")]
    Apply { scan: String },
    #[structopt(name = "scan", about = "Scan a directory for movies")]
    Scan { directory: String },
    #[structopt(name = "view", about = "View a scan result file")]
    View { scan: String },
    #[structopt(name = "sync", about = "Synchronize changes made on disk to the library")]
    Sync,
}

fn main() -> Result<()> {
    let input = input::Input::new();

    let mut library = open_library(&input);
    library.commit()?;

    let index = Index::load_or_create_index(".index")?;

    let args = App::from_args();

    match args {
        App::Apply { scan } => {
            cmd::apply::cmd_apply(&scan, &index, &mut library)?;
        }
        App::Scan { directory } => {
            cmd::scan::cmd_scan(&directory, &index, &library)?;
        }
        App::View { scan } => {
            cmd::view::cmd_view(&scan, &index)?;
        }
        App::Sync => {
            cmd::sync::cmd_sync(&mut library)?;
        }
    }

    Ok(())
}

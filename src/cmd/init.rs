use std::fs::DirBuilder;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::Result;
use crate::mero::Library;

pub fn cmd_init(root_path: impl Into<PathBuf>) -> Result {
    let root_path = root_path.into();

    let config = Config::open()?;

    match config {
        Some(_) => {
            println!("Configuration has already been initialized.");
        }
        None => {
            let config = Config::new(&root_path);
            DirBuilder::new().recursive(true).create(&config.meta_dir())?;
            let library_path = config.library_path();
            if !library_path.exists() {
                println!("Library not found, creating.");
                let lib = Library::create(library_path);
                lib.commit()?;
            } else {
                println!("Using existing library.");
            }
            println!("Saving config with root path {}", root_path.display());
            config.save()?;
        }
    }
    Ok(())
}

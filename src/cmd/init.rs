use std::fs::DirBuilder;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::Result;

pub fn cmd_init(root_path: impl Into<PathBuf>, force: bool) -> Result {
    let root_path = root_path.into();

    let config = Config::open()?;

    if config.is_some() && !force {
        println!("Configuration has already been initialized.");
    } else {
        let config = Config::new(&root_path);
        DirBuilder::new().recursive(true).create(&config.meta_dir())?;
        println!("Saving config with root path {}", root_path.display());
        config.save()?;
    }
    Ok(())
}

use std::path::PathBuf;

use crate::config::Config;
use crate::local_storage::LocalStorage;
use crate::mero::Result;

pub fn cmd_ignore_add(
    config: &Config,
    mut local_storage: LocalStorage,
    paths: impl IntoIterator<Item = PathBuf>,
) -> Result {
    for path in paths.into_iter() {
        local_storage.ignored.insert(path.canonicalize()?);
    }
    local_storage.save(config.local_storage_path())?;
    Ok(())
}

pub fn cmd_ignore_remove(
    config: &Config,
    mut local_storage: LocalStorage,
    paths: impl IntoIterator<Item = PathBuf>,
) -> Result {
    for path in paths.into_iter() {
        local_storage.ignored.remove(&path.canonicalize()?);
    }
    local_storage.save(config.local_storage_path())?;
    Ok(())
}

pub fn cmd_ignore_list(local_storage: LocalStorage) {
    for path in local_storage.ignored {
        println!("{}", path.display());
    }
}

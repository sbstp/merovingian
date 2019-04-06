use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::mero::Result;
use crate::utils;

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalStorage {
    pub ignored: BTreeSet<PathBuf>,
}

impl LocalStorage {
    pub fn open(path: impl AsRef<Path>) -> Result<LocalStorage> {
        let path = path.as_ref();
        if path.exists() {
            Ok(utils::deserialize_bin_gz(path)?)
        } else {
            Ok(LocalStorage {
                ignored: BTreeSet::new(),
            })
        }
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result {
        utils::serialize_bin_gz(path, &self)?;
        Ok(())
    }
}

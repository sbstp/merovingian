use std::collections::HashSet;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::index::TitleId;
use crate::io::Fingerprint;
use crate::scan::MovieIdentity;
use crate::utils;

#[derive(Deserialize, Serialize, Eq, PartialEq, Clone, Debug)]
pub struct RelativePath(PathBuf);

impl RelativePath {
    pub fn new(path: impl Into<PathBuf>) -> RelativePath {
        let path = path.into();
        if path.is_absolute() {
            panic!("relative path created with absolute path");
        }
        RelativePath(path)
    }

    pub fn with_root(root: impl AsRef<Path>, path: impl AsRef<Path>) -> RelativePath {
        let root = root.as_ref();
        let path = path.as_ref();
        let rel_path = path.strip_prefix(root).expect("path does not start with root");
        RelativePath::new(rel_path)
    }
}

impl Deref for RelativePath {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for RelativePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(Deserialize, Serialize)]
pub struct Subtitle {
    pub path: RelativePath,
    pub fingerprint: Fingerprint,
}

#[derive(Deserialize, Serialize)]
pub struct Movie {
    pub path: RelativePath,
    pub fingerprint: Fingerprint,
    pub subtitles: Vec<Subtitle>,
    pub images: Vec<RelativePath>,
    pub identity: MovieIdentity,
}

#[derive(Deserialize, Serialize)]
pub struct Content {
    pub movies: Vec<Movie>,
}

pub struct Library {
    path: PathBuf,
    content: Content,
    fingerprints: HashSet<Fingerprint>,
    titles: HashSet<TitleId>,
}

impl Library {
    pub fn create(path: impl Into<PathBuf>) -> Library {
        Library {
            path: path.into(),
            content: Content { movies: vec![] },
            fingerprints: HashSet::new(),
            titles: HashSet::new(),
        }
    }

    pub fn open(path: impl Into<PathBuf>) -> Result<Library> {
        let path = path.into();
        let content = utils::deserialize_bin_gz(&path)?;

        let mut library = Library {
            path: path.into(),
            content: content,
            fingerprints: HashSet::new(),
            titles: HashSet::new(),
        };

        library.rebuild_index();

        Ok(library)
    }

    pub fn movies(&self) -> &[Movie] {
        &self.content.movies
    }

    pub fn commit(&self) -> Result<()> {
        utils::serialize_bin_gz(&self.path, &self.content)
    }

    #[inline]
    pub fn has_fingerprint(&self, fingerprint: &Fingerprint) -> bool {
        self.fingerprints.contains(fingerprint)
    }

    #[inline]
    pub fn has_title(&self, title_id: TitleId) -> bool {
        self.titles.contains(&title_id)
    }

    pub fn add_movie(
        &mut self,
        identity: &MovieIdentity,
        path: RelativePath,
        fingeprint: Fingerprint,
        subtitles: impl Into<Vec<Subtitle>>,
    ) {
        self.fingerprints.insert(fingeprint.clone());
        self.titles.insert(identity.title.title_id);

        self.content.movies.push(Movie {
            path: path,
            fingerprint: fingeprint,
            subtitles: subtitles.into(),
            images: vec![],
            identity: identity.clone(),
        });
    }

    fn rebuild_index(&mut self) {
        self.fingerprints.clear();
        self.titles.clear();

        for movie in self.content.movies.iter() {
            self.fingerprints.insert(movie.fingerprint.clone());
            self.titles.insert(movie.identity.title.title_id);
        }
    }
}

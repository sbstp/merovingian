use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{Result};
use crate::io::Fingerprint;
use crate::index::TitleId;
use crate::scan::MovieIdentity;
use crate::utils::{self, VecAccess, VecAccessKey, VecAccessKeyIter};

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

    pub fn movie_access_keys(&self) -> VecAccessKeyIter {
        self.content.movies.access_keys()
    }

    pub fn movie(&self, key: VecAccessKey) -> &Movie {
        &self.content.movies.access(key)
    }

    pub fn movies(&self) -> &[Movie] {
        &self.content.movies
    }

    pub fn movie_mut(&mut self, key: VecAccessKey) -> MovieMutGuard {
        let movie = self.content.movies.access(key);

        self.fingerprints.remove(&movie.fingerprint);
        self.titles.remove(&movie.identity.title.title_id);

        MovieMutGuard {
            library: self,
            key: key,
        }
    }

    pub fn movies_mut(&mut self) -> MoviesMutGuard {
        MoviesMutGuard(self)
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

/// Guard meant to update the fingerprints and and titles sets after updating a movie.
pub struct MovieMutGuard<'l> {
    library: &'l mut Library,
    key: VecAccessKey,
}

impl Deref for MovieMutGuard<'_> {
    type Target = Movie;

    #[inline]
    fn deref(&self) -> &Movie {
        self.library.content.movies.access(self.key)
    }
}

impl DerefMut for MovieMutGuard<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Movie {
        self.library.content.movies.access_mut(self.key)
    }
}

impl Drop for MovieMutGuard<'_> {
    #[inline]
    fn drop(&mut self) {
        self.library.fingerprints.insert(self.fingerprint.clone());
        self.library.titles.insert(self.identity.title.title_id);
    }
}

pub struct MoviesMutGuard<'l>(&'l mut Library);

impl Deref for MoviesMutGuard<'_> {
    type Target = Vec<Movie>;

    #[inline]
    fn deref(&self) -> &Vec<Movie> {
        &self.0.content.movies
    }
}

impl DerefMut for MoviesMutGuard<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Vec<Movie> {
        &mut self.0.content.movies
    }
}

impl Drop for MoviesMutGuard<'_> {
    #[inline]
    fn drop(&mut self) {
        self.0.rebuild_index();
    }
}

#[cfg(test)]
mod tests {
    use crate::index::{Title, TitleId};
    use crate::io::Fingerprint;
    use crate::mero::{Library, RelativePath};
    use crate::scan::MovieIdentity;
    use crate::service::tmdb;

    fn make_dummy_identity() -> MovieIdentity {
        MovieIdentity {
            title: Title {
                title_id: TitleId::new(100),
                primary_title: String::new(),
                original_title: None,
                year: 2010,
                runtime: 120,
                vote_count: 5000,
            },
            tmdb_title: tmdb::Title {
                id: 100,
                title: String::new(),
                original_title: String::new(),
                original_language: String::new(),
                overview: String::new(),
                release_date: String::new(),
                popularity: 1.0,
                vote_count: 1000,
                vote_average: 8.0,
                poster_path: None,
                backdrop_path: None,
                adult: false,
            },
        }
    }

    #[test]
    fn test_movie_mut() {
        let mut lib = Library::create("lib.json");

        lib.add_movie(
            &make_dummy_identity(),
            RelativePath::new("foo.mkv"),
            Fingerprint::null(),
            vec![],
        );

        for key in lib.movie_access_keys() {
            lib.movie_mut(key).identity.title.title_id = TitleId::new(200);
        }

        assert!(lib.has_title(TitleId::new(200)));
    }

    #[test]
    fn test_movies_mut() {
        let mut lib = Library::create("lib.json");

        lib.add_movie(
            &make_dummy_identity(),
            RelativePath::new("foo.mkv"),
            Fingerprint::null(),
            vec![],
        );

        for movie in lib.movies_mut().iter_mut() {
            movie.identity.title.title_id = TitleId::new(200);
        }

        assert!(lib.has_title(TitleId::new(200)));
    }

}

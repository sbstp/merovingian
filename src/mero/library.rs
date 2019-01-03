use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use hashbrown::HashSet;
use serde::{Deserialize, Serialize};

use super::{index, Fingerprint, Result};

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
    pub title_id: u32,
    pub path: RelativePath,
    pub subtitles: Vec<Subtitle>,
    pub fingerprint: Fingerprint,
}

#[derive(Deserialize, Serialize)]
pub struct Content {
    pub movies: Vec<Movie>,
}

pub struct MoviesMut<'l>(&'l mut Library);

impl Deref for MoviesMut<'_> {
    type Target = Vec<Movie>;

    #[inline]
    fn deref(&self) -> &Vec<Movie> {
        &self.0.content.movies
    }
}

impl DerefMut for MoviesMut<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Vec<Movie> {
        &mut self.0.content.movies
    }
}

impl Drop for MoviesMut<'_> {
    #[inline]
    fn drop(&mut self) {
        self.0.rebuild_index();
    }
}

pub struct Library {
    path: PathBuf,
    content: Content,
    fingerprints: HashSet<Fingerprint>,
    titles: HashSet<u32>,
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
        let file = BufReader::new(File::open(&path)?);

        let mut library = Library {
            path: path.into(),
            content: serde_json::from_reader(file)?,
            fingerprints: HashSet::new(),
            titles: HashSet::new(),
        };

        library.rebuild_index();

        Ok(library)
    }

    #[inline]
    pub fn movies(&self) -> &[Movie] {
        &self.content.movies
    }

    #[inline]
    pub fn movies_mut(&mut self) -> MoviesMut {
        MoviesMut(self)
    }

    pub fn commit(&self) -> Result<()> {
        let file = BufWriter::new(File::create(&self.path)?);
        serde_json::to_writer_pretty(file, &self.content)?;
        Ok(())
    }

    #[inline]
    pub fn has_fingerprint(&self, fingerprint: &Fingerprint) -> bool {
        self.fingerprints.contains(fingerprint)
    }

    #[inline]
    pub fn has_title(&self, title_id: u32) -> bool {
        self.titles.contains(&title_id)
    }

    pub fn add_movie(
        &mut self,
        title: &index::Title,
        path: RelativePath,
        fingeprint: Fingerprint,
        subtitles: impl Into<Vec<Subtitle>>,
    ) {
        self.fingerprints.insert(fingeprint.clone());
        self.titles.insert(title.title_id);

        self.content.movies.push(Movie {
            title_id: title.title_id,
            path: path,
            fingerprint: fingeprint,
            subtitles: subtitles.into(),
        });
    }

    fn rebuild_index(&mut self) {
        self.fingerprints.clear();
        self.titles.clear();

        for movie in self.content.movies.iter() {
            self.fingerprints.insert(movie.fingerprint.clone());
            self.titles.insert(movie.title_id);
        }
    }
}

#[test]
fn test_consistent_sets() {
    let mut lib = Library::create("lib.json");
    let title = index::Title {
        title_id: 100,
        primary_title: String::new(),
        original_title: None,
        year: 2010,
        runtime: 120,
        vote_count: 5000,
    };
    lib.add_movie(&title, RelativePath::new("foo.mkv"), Fingerprint::null(), vec![]);

    for movie in lib.movies_mut().iter_mut() {
        movie.title_id = 200;
    }

    assert!(lib.has_title(200));
}

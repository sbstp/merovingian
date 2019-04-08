use std::fmt;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AbsPath(PathBuf);

impl AbsPath {
    pub fn new(path: impl Into<PathBuf>) -> Option<AbsPath> {
        let buf = path.into();
        if !buf.is_absolute() {
            panic!("given path is not absolute");
        }
        if buf.to_str().is_some() {
            Some(AbsPath(buf))
        } else {
            None
        }
    }

    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap()
    }

    pub fn file_name(&self) -> &str {
        self.0.file_name().map(|s| s.to_str().unwrap()).unwrap_or("")
    }

    pub fn file_stem(&self) -> &str {
        self.0.file_stem().map(|s| s.to_str().unwrap()).unwrap_or("")
    }

    pub fn extension(&self) -> &str {
        self.0.extension().map(|s| s.to_str().unwrap()).unwrap_or("")
    }
}

impl Deref for AbsPath {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

impl DerefMut for AbsPath {
    fn deref_mut(&mut self) -> &mut PathBuf {
        &mut self.0
    }
}

impl AsRef<Path> for AbsPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<str> for AbsPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AbsPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelPath(PathBuf);

impl RelPath {
    pub fn new(path: impl Into<PathBuf>) -> Option<RelPath> {
        let buf = path.into();
        if buf.is_absolute() {
            panic!("given path is absolute");
        }
        if buf.to_str().is_some() {
            Some(RelPath(buf))
        } else {
            None
        }
    }

    pub fn from_string(path: String) -> RelPath {
        let mut buf = PathBuf::new();
        buf.push(path);

        if buf.is_absolute() {
            panic!("given path is absolute");
        }

        RelPath(buf)
    }

    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }

    pub fn as_str(&self) -> &str {
        self.0.to_str().unwrap()
    }

    pub fn file_name(&self) -> &str {
        self.0.file_name().map(|s| s.to_str().unwrap()).unwrap_or("")
    }

    pub fn file_stem(&self) -> &str {
        self.0.file_stem().map(|s| s.to_str().unwrap()).unwrap_or("")
    }

    pub fn extension(&self) -> &str {
        self.0.extension().map(|s| s.to_str().unwrap()).unwrap_or("")
    }
}

impl Deref for RelPath {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        &self.0
    }
}

impl DerefMut for RelPath {
    fn deref_mut(&mut self) -> &mut PathBuf {
        &mut self.0
    }
}

impl AsRef<Path> for RelPath {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<str> for RelPath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.display().fmt(f)
    }
}

#[test]
fn test_rel_path_new() {
    RelPath::new("foo.txt").unwrap();
}

#[test]
#[should_panic]
fn test_rel_path_new_fail() {
    RelPath::new("/foo.txt").unwrap();
}

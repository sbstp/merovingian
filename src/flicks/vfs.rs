use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::fs::Metadata;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::tree::{self, NodeId, Tree};

struct FileNode {
    path: PathBuf,
    metadata: Metadata,
}

#[derive(Clone)]
pub struct File {
    tree: Rc<Tree<FileNode>>,
    node: NodeId,
}

impl PartialEq for File {
    fn eq(&self, other: &File) -> bool {
        self.node == other.node
    }
}

impl Eq for File {}

impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node.hash(state);
    }
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "File[{}]", self.path().display())
    }
}

impl File {
    fn data(&self) -> &FileNode {
        self.tree.data(self.node)
    }

    pub fn path(&self) -> &Path {
        &self.data().path
    }

    pub fn name(&self) -> &str {
        self.path().file_name().and_then(|s| s.to_str()).unwrap_or("")
    }

    pub fn stem(&self) -> &str {
        self.path().file_stem().and_then(|s| s.to_str()).unwrap_or("")
    }

    pub fn ext(&self) -> &str {
        self.path().extension().and_then(|s| s.to_str()).unwrap_or("")
    }

    pub fn metadata(&self) -> &Metadata {
        &self.data().metadata
    }

    pub fn is_file(&self) -> bool {
        self.metadata().is_file()
    }

    pub fn is_dir(&self) -> bool {
        self.metadata().is_dir()
    }

    pub fn len(&self) -> u64 {
        self.metadata().len()
    }

    pub fn parent(&self) -> Option<File> {
        self.tree.parent(self.node).map(|node| File {
            tree: self.tree.clone(),
            node: node,
        })
    }

    pub fn children<'a>(&'a self) -> ChildrenIter {
        ChildrenIter {
            tree: self.tree.clone(),
            iter: self.tree.children(self.node),
        }
    }

    pub fn siblings<'a>(&'a self) -> SiblingsIter {
        SiblingsIter {
            tree: self.tree.clone(),
            iter: self.tree.siblings(self.node),
        }
    }

    pub fn descendants<'a>(&'a self) -> DescendantsIter {
        DescendantsIter {
            tree: self.tree.clone(),
            iter: self.tree.descendants(self.node),
        }
    }
}

pub struct ChildrenIter<'a> {
    tree: Rc<Tree<FileNode>>,
    iter: tree::ChildrenIter<'a, FileNode>,
}

impl Iterator for ChildrenIter<'_> {
    type Item = File;

    fn next(&mut self) -> Option<File> {
        self.iter.next().map(|node| File {
            tree: self.tree.clone(),
            node: node,
        })
    }
}

pub struct SiblingsIter<'a> {
    tree: Rc<Tree<FileNode>>,
    iter: tree::SiblingsIter<'a, FileNode>,
}

impl Iterator for SiblingsIter<'_> {
    type Item = File;

    fn next(&mut self) -> Option<File> {
        self.iter.next().map(|node| File {
            tree: self.tree.clone(),
            node: node,
        })
    }
}

pub struct DescendantsIter<'a> {
    tree: Rc<Tree<FileNode>>,
    iter: tree::DescendantsIter<'a, FileNode>,
}

impl Iterator for DescendantsIter<'_> {
    type Item = File;

    fn next(&mut self) -> Option<File> {
        self.iter.next().map(|node| File {
            tree: self.tree.clone(),
            node: node,
        })
    }
}

pub fn walk(root: impl AsRef<Path>) -> io::Result<File> {
    let mut tree = Tree::new();

    match walk_rec(&mut tree, root.as_ref().to_owned(), None)? {
        Some(node) => Ok(File {
            tree: Rc::new(tree),
            node,
        }),
        None => Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid root path")),
    }
}

fn walk_rec(tree: &mut Tree<FileNode>, path: PathBuf, parent: Option<NodeId>) -> io::Result<Option<NodeId>> {
    let metadata = path.metadata()?;

    if !metadata.is_dir() && !metadata.is_file() {
        return Ok(None);
    }

    let file_node = FileNode { path: path, metadata };

    let node = match parent {
        Some(parent) => tree.insert_below(parent, file_node),
        None => tree.insert_root(file_node),
    };

    let file_node = tree.data(node);

    if file_node.metadata.is_dir() {
        for entry in file_node.path.read_dir()? {
            let entry = entry?;
            walk_rec(tree, entry.path(), Some(node))?;
        }
    }

    Ok(Some(node))
}

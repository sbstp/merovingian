use std::fs::Metadata;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use super::tree::{self, NodeId, Tree};

struct FileNode {
    path: PathBuf,
    metadata: Metadata,
}

pub struct File {
    tree: Rc<Tree<FileNode>>,
    node: NodeId,
}

impl File {
    fn data(&self) -> &FileNode {
        self.tree.data(self.node)
    }

    pub fn path(&self) -> &Path {
        &self.data().path
    }

    pub fn file(&self) -> Option<&str> {
        self.path().file_name().and_then(|s| s.to_str())
    }

    pub fn stem(&self) -> Option<&str> {
        self.path().file_stem().and_then(|s| s.to_str())
    }

    pub fn ext(&self) -> Option<&str> {
        self.path().extension().and_then(|s| s.to_str())
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

pub fn walk(path: impl AsRef<Path>) -> io::Result<File> {
    let mut tree = Tree::new();

    match walk_rec(&mut tree, path.as_ref().to_owned(), None)? {
        Some(node) => Ok(File {
            tree: Rc::new(tree),
            node,
        }),
        None => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid root folder",
        )),
    }
}

fn walk_rec(
    tree: &mut Tree<FileNode>,
    path: PathBuf,
    parent: Option<NodeId>,
) -> io::Result<Option<NodeId>> {
    let metadata = path.metadata()?;

    if !metadata.is_dir() && !metadata.is_file() {
        return Ok(None);
    }

    let file_node = FileNode {
        path: path,
        metadata,
    };

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

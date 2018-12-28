use std::collections::VecDeque;
use std::num::NonZeroUsize;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct NodeId(NonZeroUsize);

impl NodeId {
    fn new(id: usize) -> NodeId {
        NodeId(NonZeroUsize::new(id + 1).expect("node id with value of 0"))
    }

    #[inline]
    fn get(&self) -> usize {
        self.0.get() - 1
    }
}

#[derive(Debug, Copy, Clone)]
struct Node<T> {
    data: T,
    parent: Option<NodeId>,
    next_sibling: Option<NodeId>,
    prev_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
}

#[derive(Debug)]
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Tree<T> {
    #[inline]
    fn next_node_id(&self) -> NodeId {
        NodeId::new(self.nodes.len())
    }

    #[inline]
    fn select(&self, node: NodeId) -> &Node<T> {
        &self.nodes[node.get()]
    }

    #[inline]
    fn select_mut(&mut self, node: NodeId) -> &mut Node<T> {
        &mut self.nodes[node.get()]
    }

    pub fn new() -> Tree<T> {
        Tree { nodes: vec![] }
    }

    pub fn data(&self, node: NodeId) -> &T {
        &self.select(node).data
    }

    pub fn data_mut(&mut self, node: NodeId) -> &mut T {
        &mut self.select_mut(node).data
    }

    pub fn insert_root(&mut self, data: T) -> NodeId {
        let new_node_id = self.next_node_id();
        self.nodes.push(Node {
            data: data,
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
        });
        new_node_id
    }

    pub fn insert_below(&mut self, parent: NodeId, data: T) -> NodeId {
        let new_node_id = self.next_node_id();
        let prev_last_child = self.select(parent).last_child;

        match prev_last_child {
            Some(prev_last_child) => {
                self.select_mut(prev_last_child).next_sibling = Some(new_node_id);
                self.select_mut(parent).last_child = Some(new_node_id);

                self.nodes.push(Node {
                    data: data,
                    parent: Some(parent),
                    prev_sibling: Some(prev_last_child),
                    next_sibling: None,
                    first_child: None,
                    last_child: None,
                });
            }
            None => {
                let parent_node = self.select_mut(parent);
                parent_node.first_child = Some(new_node_id);
                parent_node.last_child = Some(new_node_id);

                self.nodes.push(Node {
                    data: data,
                    parent: Some(parent),
                    prev_sibling: None,
                    next_sibling: None,
                    first_child: None,
                    last_child: None,
                });
            }
        }

        new_node_id
    }

    pub fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.select(node).parent
    }

    pub fn children(&self, node: NodeId) -> ChildrenIter<T> {
        ChildrenIter {
            tree: self,
            current: self.select(node).first_child,
        }
    }

    pub fn siblings(&self, node: NodeId) -> SiblingsIter<T> {
        SiblingsIter {
            tree: self,
            origin: node,
            current: self.parent(node).and_then(|p| self.select(p).first_child),
        }
    }

    pub fn descendants(&self, node: NodeId) -> DescendantsIter<T> {
        DescendantsIter {
            tree: self,
            queue: self.children(node).collect::<VecDeque<NodeId>>(),
        }
    }
}

pub struct ChildrenIter<'t, T> {
    tree: &'t Tree<T>,
    current: Option<NodeId>,
}

impl<T> Iterator for ChildrenIter<'_, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        match self.current {
            None => None,
            Some(current) => {
                self.current = self.tree.select(current).next_sibling;
                Some(current)
            }
        }
    }
}

pub struct SiblingsIter<'t, T> {
    tree: &'t Tree<T>,
    current: Option<NodeId>,
    origin: NodeId,
}

impl<T> Iterator for SiblingsIter<'_, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        loop {
            match self.current {
                None => return None,
                Some(current) => {
                    self.current = self.tree.select(current).next_sibling;
                    if current != self.origin {
                        return Some(current);
                    }
                }
            }
        }
    }
}

pub struct DescendantsIter<'t, T> {
    tree: &'t Tree<T>,
    queue: VecDeque<NodeId>,
}

impl<T> Iterator for DescendantsIter<'_, T> {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        match self.queue.pop_front() {
            None => None,
            Some(node) => {
                self.queue.extend(self.tree.children(node));
                Some(node)
            }
        }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use super::Tree;

    #[test]
    fn test_parent() {
        let mut t = Tree::new();
        let root = t.insert_root("root");
        let child1 = t.insert_below(root, "child1");
        let child1_1 = t.insert_below(child1, "child1-1");
        let child1_2 = t.insert_below(child1, "child1-2");
        let child2 = t.insert_below(root, "child2");
        let child2_1 = t.insert_below(child2, "child2-1");
        let child3 = t.insert_below(root, "child3");

        assert_eq!(t.parent(root), None);
        assert_eq!(t.parent(child1), Some(root));
        assert_eq!(t.parent(child1_1), Some(child1));
    }

    #[test]
    fn test_children() {
        let mut t = Tree::new();
        let root = t.insert_root("root");
        let child1 = t.insert_below(root, "child1");
        let child1_1 = t.insert_below(child1, "child1-1");
        let child1_2 = t.insert_below(child1, "child1-2");
        let child2 = t.insert_below(root, "child2");
        let child2_1 = t.insert_below(child2, "child2-1");
        let child3 = t.insert_below(root, "child3");

        assert_eq!(t.children(root).collect::<Vec<_>>(), vec![child1, child2, child3]);
        assert_eq!(t.children(child1).collect::<Vec<_>>(), vec![child1_1, child1_2]);

        assert_eq!(t.children(child1_1).collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_siblings() {
        let mut t = Tree::new();
        let root = t.insert_root("root");
        let child1 = t.insert_below(root, "child1");
        let child1_1 = t.insert_below(child1, "child1-1");
        let child1_2 = t.insert_below(child1, "child1-2");
        let child2 = t.insert_below(root, "child2");
        let child2_1 = t.insert_below(child2, "child2-1");
        let child3 = t.insert_below(root, "child3");

        assert_eq!(t.siblings(root).collect::<Vec<_>>(), vec![],);
        assert_eq!(t.siblings(child1).collect::<Vec<_>>(), vec![child2, child3]);
        assert_eq!(t.siblings(child1_1).collect::<Vec<_>>(), vec![child1_2]);
    }

    #[test]
    fn test_descendants() {
        let mut t = Tree::new();
        let root = t.insert_root("root");
        let child1 = t.insert_below(root, "child1");
        let child1_1 = t.insert_below(child1, "child1-1");
        let child1_2 = t.insert_below(child1, "child1-2");
        let child2 = t.insert_below(root, "child2");
        let child2_1 = t.insert_below(child2, "child2-1");
        let child3 = t.insert_below(root, "child3");

        assert_eq!(
            t.descendants(root).collect::<Vec<_>>(),
            vec![child1, child2, child3, child1_1, child1_2, child2_1]
        );
        assert_eq!(t.descendants(child1_1).collect::<Vec<_>>(), vec![]);
    }
}

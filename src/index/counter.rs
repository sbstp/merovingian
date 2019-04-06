use std::hash::Hash;
use std::ops::Index;

use std::collections::{
    hash_map::{Iter, Keys, Values},
    HashMap,
};

#[derive(Debug)]
pub struct Counter<K: Hash + Eq> {
    inner: HashMap<K, u32>,
}

impl<K> Counter<K>
where
    K: Hash + Eq,
{
    pub fn new() -> Counter<K> {
        Counter { inner: HashMap::new() }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn add(&mut self, key: K) {
        *self.inner.entry(key).or_insert(0) += 1;
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = K>,
    {
        for item in iter {
            self.add(item);
        }
    }

    pub fn iter(&self) -> Iter<K, u32> {
        self.inner.iter()
    }

    pub fn keys(&self) -> Keys<K, u32> {
        self.inner.keys()
    }

    pub fn values(&self) -> Values<K, u32> {
        self.inner.values()
    }

    pub fn get(&self, key: &K) -> Option<&u32> {
        self.inner.get(key)
    }
}

impl<K> Index<&K> for Counter<K>
where
    K: Hash + Eq,
{
    type Output = u32;
    fn index(&self, key: &K) -> &u32 {
        &self.inner[key]
    }
}

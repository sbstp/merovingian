use std::cmp;
use std::hash::Hash;
use std::ops::{Deref, Index};

use hashbrown::hash_map::{Iter, Keys, Values};
use hashbrown::HashMap;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct NonNan(f64);

impl NonNan {
    pub fn new(val: f64) -> NonNan {
        if val.is_nan() {
            panic!("NonNan created with NaN value");
        }
        NonNan(val)
    }
}

impl cmp::Ord for NonNan {
    #[inline]
    fn cmp(&self, other: &NonNan) -> cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl cmp::Eq for NonNan {}

impl Deref for NonNan {
    type Target = f64;

    #[inline]
    fn deref(&self) -> &f64 {
        &self.0
    }
}

#[derive(Debug)]
pub struct Counter<K: Hash + Eq> {
    inner: HashMap<K, u32>,
}

impl<K> Counter<K>
where
    K: Hash + Eq,
{
    pub fn new() -> Counter<K> {
        Counter {
            inner: HashMap::new(),
        }
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

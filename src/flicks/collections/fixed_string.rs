use std::cmp;
use std::fmt;
use std::hash;
use std::ops;
use std::str;

use serde::{Deserialize, Serialize};

const SIZE: usize = 15;

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct FixedString {
    len: u8,
    buf: [u8; SIZE],
}

impl FixedString {
    pub fn new(source: &str) -> FixedString {
        let mut buf = [0u8; SIZE];
        let mut it = source.chars();
        let mut len = 0;

        while let Some(c) = it.next() {
            let car_len = c.len_utf8();
            if len + car_len <= SIZE {
                len += car_len;
            } else {
                break;
            }
        }

        buf[..len].copy_from_slice(&source.as_bytes()[..len]);

        FixedString { len: len as u8, buf }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.buf[..self.len as usize]) }
    }
}

impl ops::Deref for FixedString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl cmp::PartialEq for FixedString {
    fn eq(&self, other: &FixedString) -> bool {
        self.as_str() == other.as_str()
    }
}

impl cmp::PartialEq<str> for FixedString {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl cmp::Eq for FixedString {}

impl cmp::PartialOrd for FixedString {
    fn partial_cmp(&self, other: &FixedString) -> Option<cmp::Ordering> {
        Some(self.as_str().cmp(other.as_str()))
    }
}

impl cmp::PartialOrd<str> for FixedString {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        Some(self.as_str().cmp(other))
    }
}

impl cmp::Ord for FixedString {
    fn cmp(&self, other: &FixedString) -> cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl hash::Hash for FixedString {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl fmt::Display for FixedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl fmt::Debug for FixedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FixedString(\"{}\")", self.as_str())
    }
}

#[test]
fn test_new() {
    let s = FixedString::new("hello");
    assert_eq!(s.as_str(), "hello");
}

#[test]
fn test_long() {
    let s = FixedString::new("hellohellohellohello");
    assert_eq!(&s, "hellohellohello");
}

#[test]
fn test_size() {
    assert_eq!(std::mem::size_of::<FixedString>(), SIZE + 1);
}

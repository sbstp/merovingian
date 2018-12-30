use std::cmp;
use std::fmt;
use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NonNan(f64);

impl NonNan {
    pub fn new(val: f64) -> NonNan {
        if val.is_nan() {
            panic!("NonNan created with NaN value");
        }
        NonNan(val)
    }
}

impl fmt::Display for NonNan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
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

pub fn clean_path(source: &str) -> String {
    let mut dest = String::with_capacity(source.len());
    for car in source.chars() {
        dest.push(match car {
            '/' | '<' | '>' | ':' | '"' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_ascii_control() => '_',
            _ => car,
        });
    }
    let tlen = dest.trim_end_matches(&[' ', '.'][..]).len();
    dest.truncate(tlen);
    dest
}

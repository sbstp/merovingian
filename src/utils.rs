use std::cmp;
use std::ops::{Deref};

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

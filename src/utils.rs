use std::cmp;
use std::fmt;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read};
use std::ops::Deref;
use std::path::Path;

use libflate::gzip::{Decoder, Encoder};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use super::Result;

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

pub struct SafeBuffer(Vec<u8>);

impl SafeBuffer {
    pub fn new() -> SafeBuffer {
        SafeBuffer(Vec::new())
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Read an exact amount of bytes from the given reader.
    ///
    /// If the given reader does not have sufficient data, an error is returned.
    /// This operation grows the buffer.
    pub fn read_exact<R>(&mut self, mut reader: R, num: usize) -> io::Result<()>
    where
        R: Read,
    {
        let len = self.0.len();
        let new_len = len + num;

        self.0.reserve(num);
        unsafe {
            self.0.set_len(new_len);
        }

        reader.read_exact(&mut self.0[len..new_len])?;
        Ok(())
    }

    /// Read the given reader to its end.
    ///
    /// This operation grows the buffer.
    #[inline]
    pub fn read_to_end<R>(&mut self, mut reader: R) -> io::Result<usize>
    where
        R: Read,
    {
        reader.read_to_end(&mut self.0)
    }

    /// Read up to `num` bytes into the buffer.
    ///
    /// This operation clears the buffer before reading, it does not grow the buffer.
    pub fn clear_read<R>(&mut self, mut reader: R, num: usize) -> io::Result<usize>
    where
        R: Read,
    {
        self.0.clear();

        self.0.reserve(num);
        unsafe {
            self.0.set_len(num);
        }

        let n = reader.read(&mut self.0)?;

        unsafe {
            self.0.set_len(n);
        }

        Ok(n)
    }
}

pub fn serialize_bin<T: Serialize>(path: impl AsRef<Path>, obj: &T) -> Result {
    let writer = BufWriter::new(File::create(path.as_ref())?);
    bincode::serialize_into(writer, obj)?;
    Ok(())
}

pub fn serialize_bin_gz<T: Serialize>(path: impl AsRef<Path>, obj: &T) -> Result {
    let writer = Encoder::new(File::create(path.as_ref())?)?;
    bincode::serialize_into(writer, obj)?;
    Ok(())
}

pub fn deserialize_bin<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let reader = BufReader::new(File::open(path.as_ref())?);
    let obj = bincode::deserialize_from(reader)?;
    Ok(obj)
}

pub fn deserialize_bin_gz<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let reader = Decoder::new(File::open(path.as_ref())?)?;
    let obj = bincode::deserialize_from(reader)?;
    Ok(obj)
}

pub fn serialize_json<T: Serialize>(path: impl AsRef<Path>, obj: &T) -> Result {
    let writer = BufWriter::new(File::create(path.as_ref())?);
    serde_json::to_writer_pretty(writer, obj)?;
    Ok(())
}

pub fn deserialize_json<T: DeserializeOwned>(path: impl AsRef<Path>) -> Result<T> {
    let reader = BufReader::new(File::open(path.as_ref())?);
    let obj = serde_json::from_reader(reader)?;
    Ok(obj)
}

impl Deref for SafeBuffer {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0
    }
}

#[test]
fn test_safe_buffer() {
    use std::io::Cursor;

    let mut b = SafeBuffer::new();
    let mut c = Cursor::new("hello world");

    b.read_exact(&mut c, 3).unwrap();
    assert_eq!(&b[..], b"hel");

    b.read_exact(&mut c, 2).unwrap();
    assert_eq!(&b[..], b"hello");

    assert_eq!(b.read_to_end(&mut c).unwrap(), 6);
    assert_eq!(&b[..], b"hello world");
}

#[test]
fn test_safe_buffer_clear_read() {
    let mut b = SafeBuffer::new();

    b.clear_read(&b"hello"[..], 10).unwrap();
    assert_eq!(&b[..], b"hello");

    b.clear_read(&b"world"[..], 10).unwrap();
    assert_eq!(&b[..], b"world");
}

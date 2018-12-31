use std::cmp;
use std::fmt::Write;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const BLOCK_SIZE: u64 = 64 * 1024; // 64 KiB

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fingerprint(String);

fn hash(bytes: &[u8]) -> Fingerprint {
    let mut hasher = Sha256::default();

    hasher.input(&bytes);

    let mut hash = String::with_capacity(64);
    let output = &hasher.result()[..];
    for byte in output {
        let _ = write!(hash, "{:02x}", byte);
    }

    Fingerprint(hash)
}

/// Calculate the start position and length of the data to be read.
/// We want the hash to be from the middle of the data. If there isn't
/// enough data to read a full BLOCK_SIZE, the entire data is read.
#[inline]
fn calc(len: u64) -> (u64, usize) {
    if len == 0 {
        panic!("attempted to fingerprint an empty buffer")
    }
    let read_max = cmp::min(len, BLOCK_SIZE as u64) as usize;
    let seek_pos = (len / 2).checked_sub(BLOCK_SIZE / 2).unwrap_or(0);

    (seek_pos, read_max)
}

pub fn bytes(bytes: &[u8]) -> Fingerprint {
    let (seek_pos, read_max) = calc(bytes.len() as u64);
    let start = seek_pos as usize;
    let end = start + read_max;
    hash(&bytes[start..end])
}

pub fn file<A>(path: A) -> io::Result<Fingerprint>
where
    A: AsRef<Path>,
{
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();

    let (seek_pos, read_max) = calc(len);

    let mut buf = Vec::with_capacity(read_max);
    unsafe {
        buf.set_len(buf.capacity());
    }

    file.seek(SeekFrom::Start(seek_pos))?;
    file.read_exact(&mut buf[..read_max])?;

    Ok(hash(&buf[..read_max]))
}

#[test]
fn test_small() {
    println!("{:?}", std::env::current_dir());
    let path = "testdata/fingerprint/small.bin";
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    assert_eq!(bytes(&buf), file(path).unwrap());
}

#[test]
fn test_large() {
    println!("{:?}", std::env::current_dir());
    let path = "testdata/fingerprint/large.bin";
    let mut f = File::open(path).unwrap();
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    assert_eq!(bytes(&buf), file(path).unwrap());
}

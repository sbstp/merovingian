use std::fmt::Write;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use sha2::{Digest, Sha256};

const BLOCK_SIZE: u64 = 1024 * 1024; // 1 MiB

/// Very quick file fingerprinting. Takes 1 MiB from the middle of the file.
/// There are 256^1024^2 possibilities for the sample, collisions should be very low hopefully.
/// The hash itself is Sha256, it produces 32 bytes that are hexed to a 64 character string.
pub fn file<A>(path: A) -> io::Result<String>
where
    A: AsRef<Path>,
{
    let mut buf = Vec::with_capacity(BLOCK_SIZE as usize);
    unsafe {
        buf.set_len(buf.capacity());
    }
    let mut hasher = Sha256::default();
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();

    if len == 0 {
        return Err(io::Error::new(io::ErrorKind::Other, "file is empty"));
    }

    let pos = (len / 2).checked_sub(BLOCK_SIZE / 2).unwrap_or(0);
    file.seek(SeekFrom::Start(pos))?;
    file.read_exact(&mut buf)?;
    hasher.input(&buf[..]);

    let mut hash = String::with_capacity(64);
    let output = &hasher.result()[..];
    for byte in output {
        let _ = write!(hash, "{:02x}", byte);
    }
    Ok(hash)
}

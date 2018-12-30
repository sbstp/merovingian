use std::fmt;
use std::fs::{self, DirBuilder};
use std::io::{Read, Write};
use std::path::PathBuf;

use super::error::Error;

#[derive(Debug)]
pub enum Status {
    Waiting,
    Copied,
    Copying {
        copied: u64,
        len: u64,
        src: fs::File,
        dst: fs::File,
    },
    Hardlinked,
    Err(Error),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Status::*;

        const MIB: f64 = 1024.0 * 1024.0;

        match self {
            Waiting => write!(f, "Waiting"),
            Copied => write!(f, "Copied"),
            Copying { copied, len, .. } => {
                write!(f, "Copying({:.2}/{:.2} MiB)", *copied as f64 / MIB, *len as f64 / MIB)
            }
            Hardlinked => write!(f, "Hardlinked"),
            Err(err) => write!(f, "Err({})", err),
        }
    }
}

#[derive(Debug)]
pub struct Transfer {
    pub src: PathBuf,
    pub dst: PathBuf,
    pub status: Option<Status>,
}

const EXDEV: i32 = 18;

impl Transfer {
    pub fn finished(&self) -> bool {
        match self.status.as_ref().unwrap() {
            Status::Waiting | Status::Copying { .. } => false,
            _ => true,
        }
    }

    pub fn status(&self) -> &Status {
        self.status.as_ref().unwrap()
    }

    fn update_status<'b>(&self, status: Status, buf: &'b mut Vec<u8>) -> Status {
        match status {
            Status::Waiting => {
                if let Some(parent) = self.dst.parent() {
                    if let Err(err) = DirBuilder::new().recursive(true).create(parent) {
                        return Status::Err(err.into());
                    }
                }

                match fs::hard_link(&self.src, &self.dst) {
                    Ok(_) => Status::Hardlinked,
                    // TODO: check what stupid thing Windows does with hard-linking across devices
                    Err(ref err) if err.raw_os_error() == Some(EXDEV) => {
                        match (fs::File::open(&self.src), fs::File::create(&self.dst)) {
                            (Ok(src), Ok(dst)) => Status::Copying {
                                len: src.metadata().expect("could not get src len").len(),
                                src,
                                dst,
                                copied: 0,
                            },
                            (Err(err), Ok(_)) => Status::Err(Error::transfer(err, None)),
                            (Ok(_), Err(err)) => Status::Err(Error::transfer(None, err)),
                            (Err(err_src), Err(err_dst)) => Status::Err(Error::transfer(err_src, err_dst)),
                        }
                    }
                    Err(err) => Status::Err(Error::transfer(None, err)),
                }
            }
            Status::Copying {
                mut src,
                mut dst,
                copied,
                len,
            } => match src.read(buf) {
                Ok(0) => Status::Copied,
                Ok(n) => match dst.write_all(&buf[..n]) {
                    Ok(_) => Status::Copying {
                        src,
                        dst,
                        copied: copied + n as u64,
                        len,
                    },
                    Err(err) => Status::Err(Error::transfer(None, err)),
                },
                Err(err) => Status::Err(Error::transfer(err, None)),
            },
            _ => status,
        }
    }

    fn tick<'b>(&mut self, buf: &'b mut Vec<u8>) {
        let status = self.status.take().unwrap();
        self.status = Some(self.update_status(status, buf));
    }
}

pub struct Manager {
    current: usize,
    transfers: Vec<Transfer>,
    buf: Vec<u8>,
}

impl fmt::Debug for Manager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Manager")
            .field("current", &self.current)
            .field("transfers", &self.transfers)
            .finish()
    }
}

impl Manager {
    pub fn new() -> Manager {
        Manager {
            current: 0,
            transfers: vec![],
            buf: {
                let mut v = Vec::with_capacity(4096);
                unsafe {
                    v.set_len(v.capacity());
                }
                v
            },
        }
    }

    pub fn has_work(&self) -> bool {
        self.current < self.transfers.len()
    }

    pub fn transfers(&self) -> &[Transfer] {
        &self.transfers
    }

    pub fn current(&self) -> &Transfer {
        &self.transfers[self.current]
    }

    pub fn add_transfer(&mut self, src: impl Into<PathBuf>, dst: impl Into<PathBuf>) {
        self.transfers.push(Transfer {
            src: src.into(),
            dst: dst.into(),
            status: Some(Status::Waiting),
        })
    }

    pub fn tick(&mut self) {
        if self.current < self.transfers.len() {
            let request = &mut self.transfers[self.current];
            request.tick(&mut self.buf);
            // If the request is finished, move to the next one.
            if request.finished() {
                self.current += 1;
            }
        }
    }
}

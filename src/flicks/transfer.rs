use std::fmt;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

use super::error::Error;

#[derive(Debug)]
pub enum Status {
    Waiting,
    Copied,
    Copying {
        copied: u64,
        src: fs::File,
        dst: fs::File,
    },
    Hardlinked,
    Err(Error),
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

    fn tick(&mut self, buf: &mut Vec<u8>) {
        let status = self.status.take().unwrap();
        self.status = Some(match status {
            Status::Waiting => {
                match fs::hard_link(&self.src, &self.dst) {
                    Ok(_) => Status::Hardlinked,
                    // TODO: check what stupid thing Windows does with hard-linking across devices
                    Err(ref err) if err.raw_os_error() == Some(EXDEV) => {
                        match (fs::File::open(&self.src), fs::File::open(&self.dst)) {
                            (Ok(src), Ok(dst)) => Status::Copying {
                                src,
                                dst,
                                copied: 0,
                            },
                            (Err(err), Ok(_)) => Status::Err(Error::transfer(err, None)),
                            (Ok(_), Err(err)) => Status::Err(Error::transfer(None, err)),
                            (Err(err_src), Err(err_dst)) => {
                                Status::Err(Error::transfer(err_src, err_dst))
                            }
                        }
                    }
                    Err(err) => Status::Err(Error::transfer(None, err)),
                }
            }
            Status::Copying {
                mut src,
                mut dst,
                copied,
            } => match src.read(buf) {
                Ok(n) => match dst.write_all(&buf[..n]) {
                    Ok(_) => Status::Copying {
                        src,
                        dst,
                        copied: copied + n as u64,
                    },
                    Err(err) => Status::Err(Error::transfer(None, err)),
                },
                Err(err) => Status::Err(Error::transfer(err, None)),
            },
            _ => status,
        });
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

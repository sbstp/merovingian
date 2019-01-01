use std::fmt;
use std::fs::{self, DirBuilder};
use std::io::Write;
use std::path::PathBuf;

use super::{Result, SafeBuffer};

#[derive(Debug)]
pub enum Status {
    Waiting,
    Cancelled,
    Copied,
    Copying {
        copied: u64,
        len: u64,
        src: fs::File,
        dst: fs::File,
    },
    Hardlinked,
    Err,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Status::*;

        const MIB: f64 = 1024.0 * 1024.0;

        match self {
            Waiting => write!(f, "Waiting"),
            Cancelled => write!(f, "Cancelled"),
            Copied => write!(f, "Copied"),
            Copying { copied, len, .. } => {
                write!(f, "Copying({:.2}/{:.2} MiB)", *copied as f64 / MIB, *len as f64 / MIB)
            }
            Hardlinked => write!(f, "Hardlinked"),
            Err => write!(f, "Err"),
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
    #[inline]
    pub fn status(&self) -> &Status {
        self.status.as_ref().unwrap()
    }

    #[inline]
    pub fn set_status(&mut self, status: Status) {
        self.status = Some(status);
    }

    #[inline]
    fn finished(&self) -> bool {
        match self.status() {
            Status::Waiting | Status::Copying { .. } => false,
            _ => true,
        }
    }

    fn update_status<'b>(&self, status: Status, buff: &'b mut SafeBuffer) -> Result<Status> {
        match status {
            Status::Waiting => {
                if let Some(parent) = self.dst.parent() {
                    DirBuilder::new().recursive(true).create(parent)?;
                }

                let src_metadata = self.src.metadata()?;

                // If the destination exists but doesn't have the same length,
                // as the source we might have an incomplete file. Delete it.
                if let Ok(dst_metadata) = self.dst.metadata() {
                    if dst_metadata.len() != src_metadata.len() {
                        fs::remove_file(&self.dst)?;
                    } else {
                        return Ok(Status::Copied);
                    }
                }

                // Try to hard-link the file. If it cannot be hard-linked, copy it.
                Ok(match fs::hard_link(&self.src, &self.dst) {
                    Ok(_) => Status::Hardlinked,
                    // TODO: check what stupid thing Windows does with hard-linking across devices
                    Err(ref err) if err.raw_os_error() == Some(EXDEV) => {
                        let src = fs::File::open(&self.src)?;
                        let dst = fs::File::create(&self.dst)?;

                        Status::Copying {
                            src,
                            dst,
                            len: src_metadata.len(),
                            copied: 0,
                        }
                    }
                    Err(err) => return Err(err.into()),
                })
            }
            Status::Copying {
                mut src,
                mut dst,
                copied,
                len,
            } => Ok(match buff.clear_read(&mut src, 8192)? {
                0 => Status::Copied,
                n => {
                    dst.write_all(&buff)?;
                    Status::Copying {
                        src,
                        dst,
                        len,
                        copied: copied + n as u64,
                    }
                }
            }),

            _ => Ok(status),
        }
    }

    #[inline]
    fn step<'b>(&mut self, buff: &'b mut SafeBuffer) -> Result {
        let status = self.status.take().unwrap();
        match self.update_status(status, buff) {
            Ok(status) => {
                self.set_status(status);
                Ok(())
            }
            Err(err) => {
                self.set_status(Status::Err);
                Err(err)
            }
        }
    }
}

pub struct Manager {
    current: usize,
    transfers: Vec<Transfer>,
    buff: SafeBuffer,
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
            buff: SafeBuffer::new(),
        }
    }

    #[inline]
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

    pub fn step(&mut self) -> Result<Option<&Transfer>> {
        if self.current < self.transfers.len() {
            let request = &mut self.transfers[self.current];
            request.step(&mut self.buff)?;
            // If the request is finished, move to the next one.
            if request.finished() {
                self.current += 1;
            }

            Ok(self.transfers.get(self.current))
        } else {
            Ok(None)
        }
    }

    pub fn try_cancel(&mut self) {
        for transfer in self.transfers.iter_mut() {
            // Set the status to cancelled so that any open files are dropped and the remove call has no issue.
            transfer.set_status(Status::Cancelled);
            let _ = fs::remove_file(&transfer.dst);
        }
    }
}

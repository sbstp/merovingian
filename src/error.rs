use std::error;
use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::result;

use bincode;
use csv;

#[derive(Debug)]
pub enum Error {
    Bincode(bincode::Error),
    Csv(csv::Error),
    Io(io::Error),
    Json(serde_json::Error),
    ParseIntError(ParseIntError),
    Reqwest(reqwest::Error),
    SpawnError(String),
}

pub type Result<T> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Bincode(e) => write!(w, "Error({})", e),
            Error::Csv(e) => write!(w, "Error({})", e),
            Error::Io(e) => write!(w, "Error({})", e),
            Error::Json(e) => write!(w, "Error({})", e),
            Error::ParseIntError(e) => write!(w, "Error({})", e),
            Error::Reqwest(e) => write!(w, "Error({})", e),
            Error::SpawnError(e) => write!(w, "Error(SpawnError({}))", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::Bincode(e) => e.description(),
            Error::Csv(e) => e.description(),
            Error::Io(e) => e.description(),
            Error::Json(e) => e.description(),
            Error::ParseIntError(e) => e.description(),
            Error::Reqwest(e) => e.description(),
            Error::SpawnError(_) => "error spawning process",
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Bincode(e) => e.source(),
            Error::Csv(e) => e.source(),
            Error::Io(e) => e.source(),
            Error::Json(e) => e.source(),
            Error::ParseIntError(e) => e.source(),
            Error::Reqwest(e) => e.source(),
            Error::SpawnError(_) => None,
        }
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Error {
        Error::Bincode(err)
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::Csv(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Error::Json(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        Error::ParseIntError(err)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Reqwest(err)
    }
}

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
    Http(attohttpc::Error),
    SpawnError(String),
    Transfer {
        src: Option<io::Error>,
        dst: Option<io::Error>,
    },
}

pub type Result<T = ()> = result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            Bincode(e) => write!(w, "Error({})", e),
            Csv(e) => write!(w, "Error({})", e),
            Io(e) => write!(w, "Error({})", e),
            Json(e) => write!(w, "Error({})", e),
            ParseIntError(e) => write!(w, "Error({})", e),
            Http(e) => write!(w, "Error({})", e),
            SpawnError(e) => write!(w, "Error(SpawnError({}))", e),
            Transfer { src, dst } => match (src, dst) {
                (Some(e1), Some(e2)) => write!(w, "Error(Transfer(Both({}, {})))", e1, e2),
                (Some(e), _) => write!(w, "Error(Transfer(Source({})))", e),
                (_, Some(e)) => write!(w, "Error(Transfer(Destination({})))", e),
                _ => unreachable!(),
            },
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;

        match self {
            Bincode(e) => e.description(),
            Csv(e) => e.description(),
            Io(e) => e.description(),
            Json(e) => e.description(),
            ParseIntError(e) => e.description(),
            Http(e) => e.description(),
            SpawnError(_) => "error spawning process",
            Transfer { src, dst } => match (src, dst) {
                (Some(_), Some(_)) => "transfer error both source and destination",
                (Some(_), _) => "transfer error source",
                (_, Some(_)) => "transfer error destination",
                _ => unreachable!(),
            },
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use self::Error::*;

        match self {
            Bincode(e) => e.source(),
            Csv(e) => e.source(),
            Io(e) => e.source(),
            Json(e) => e.source(),
            ParseIntError(e) => e.source(),
            Http(e) => e.source(),
            SpawnError(_) => None,
            Transfer { src, dst } => match (src, dst) {
                (Some(_), Some(_)) => None,
                (Some(e), _) => e.source(),
                (_, Some(e)) => e.source(),
                _ => unreachable!(),
            },
        }
    }
}

impl Error {
    pub fn transfer(src: impl Into<Option<io::Error>>, dst: impl Into<Option<io::Error>>) -> Error {
        Error::Transfer {
            src: src.into(),
            dst: dst.into(),
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

impl From<attohttpc::Error> for Error {
    fn from(err: attohttpc::Error) -> Error {
        Error::Http(err)
    }
}

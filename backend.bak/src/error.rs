use tungstenite;

use std::io;
use std::fmt::{self, Display, Formatter};
use std::result;

pub type Result<T> = result::Result<T, Error>;

/// Error type for errors within class-queue.
#[derive(Debug)]
pub enum Error {
    /// An IO error has occured.
    Io(io::Error),
    /// A websocket protocol error has occured.
    Websocket(tungstenite::Error),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<tungstenite::Error> for Error {
    fn from(error: tungstenite::Error) -> Self {
        Error::Websocket(error)
    }
}

impl Display for Error {
    fn fmt<'a>(&self, f: &mut Formatter<'a>) -> fmt::Result {
        match self {
            &Error::Io(ref error) => write!(f, "{}", error),
            &Error::Websocket(ref error) => write!(f, "{}", error),
        }
    }
}

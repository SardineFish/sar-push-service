use std::{io, str::Utf8Error};

use crate::reply::{ParseError, Reply};

pub enum Error {
    IOError(io::Error),
    ParseError(ParseError),
    ErrorReply(Reply),
    HandshakeError(Box<Error>),
}
pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IOError(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::ParseError(err)
    }
}
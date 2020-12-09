use std::{io, net::TcpStream};

use openssl::ssl::HandshakeError;

use crate::reply::{ParseError, Reply};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    ParseError(ParseError),
    ErrorReply(Reply),
    HandshakeError(Box<Error>),
    ExtensionNotSupported(String),
    OtherError(String),
    OpenSSLError(openssl::error::ErrorStack),
    TLSHandshakeError(HandshakeError<TcpStream>),
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

impl From<openssl::error::ErrorStack> for Error{
    fn from(err: openssl::error::ErrorStack) -> Self {
        Error::OpenSSLError(err)
    }
}

impl From<HandshakeError<TcpStream>> for Error {
    fn from(err: HandshakeError<TcpStream>) -> Self {
        Error::TLSHandshakeError(err)
    }
}
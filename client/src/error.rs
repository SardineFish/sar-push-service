use reqwest::StatusCode;

pub enum Error {
    NetworkError(reqwest::Error),
    JsonError(serde_json::Error),
    ResponseError(StatusCode, String),
    IOError(std::io::Error),
    ErrorInfo(&'static str),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::NetworkError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}


pub type Result<T: Sized> = std::result::Result<T, Error>;
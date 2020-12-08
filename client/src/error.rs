use reqwest::StatusCode;

pub enum Error {
    NetworkError(reqwest::Error),
    JsonError(serde_json::Error),
    ResponseError(StatusCode, String),
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


pub type Result<T: Sized> = std::result::Result<T, Error>;
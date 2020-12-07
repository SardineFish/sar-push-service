use std::fmt::{Display};

use mongodb::bson;

#[derive(Debug)]
pub enum Error {
    MongoError(mongodb::error::Error),
    BsonDeserializeError(bson::de::Error),
    SerializeError(bson::ser::Error),
    DocError(bson::document::ValueAccessError),
    NoRecord,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Internal db error.")
    }
}

impl From<bson::ser::Error> for Error {
    fn from(err: bson::ser::Error) -> Self {
        Error::SerializeError(err)
    }
}

impl From<bson::de::Error> for Error {
    fn from(err: bson::de::Error) -> Self {
        Error::BsonDeserializeError(err)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(err: mongodb::error::Error) -> Self {
        Error::MongoError(err)
    }
}

pub fn mongo_error(err: mongodb::error::Error) -> Error {
    Error::MongoError(err)
}
pub fn bson_de_error(err: bson::de::Error) -> Error {
    Error::BsonDeserializeError(err)
}
pub fn doc_error(err: bson::document::ValueAccessError) -> Error {
    Error::DocError(err)
}
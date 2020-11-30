use mongodb::bson;

pub enum Error {
    MongoError(mongodb::error::Error),
    BsonDeserializeError(bson::de::Error),
    DocError(bson::document::ValueAccessError),
    NoRecord,
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
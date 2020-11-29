use std::iter::FilterMap;

use super::Model;

use mongodb::{
    bson,
    bson::{ doc, Bson, oid::ObjectId, },
    Cursor,
};
use serde::{Serialize, Deserialize};
use tokio::stream::StreamExt;

pub enum Error {
    MongoError(mongodb::error::Error),
    BsonDeserializeError(bson::de::Error),
    NoRecord,
}

#[derive(Serialize, Deserialize)]
pub enum Access {
    Amin,
    User,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub owner: String,
    pub secret: String,
    pub access: Access,
}

const COLLECTION_ACCESS: &str = "access";

fn mongo_error(err: mongodb::error::Error) -> Error {
    Error::MongoError(err)
}
fn bson_de_error(err: bson::de::Error) -> Error {
    Error::BsonDeserializeError(err)
}

impl Model {
    pub async fn get_all_user(&self) -> Result<Vec<User>, Error> {
        let collection = self.db.collection(COLLECTION_ACCESS);
        let cursor = collection.find(doc! {}, None).await.map_err(mongo_error)?;

        let user_list: Vec<User> = cursor.filter_map(|result| {
            if let Ok(doc) = result {
                if let Ok(user) = bson::from_document::<User>(doc) {
                    Some(user)
                } else {
                    None
                }
            } else {
                None
            }
        }).collect().await;

        Ok(user_list)
    }

    pub async fn add_user(&self, user: User) -> Result<ObjectId, Error> {
        let collection = self.db.collection(COLLECTION_ACCESS);
        let doc = mongodb::bson::to_document(&user).unwrap();
        let result = collection.insert_one(doc, None).await.map_err(mongo_error)?;
        Ok(result.inserted_id.as_object_id().unwrap().clone())
    }

    pub async fn set_user_access(&self, id: String, access: Access) -> Result<(), Error> {
        let collection = self.db.collection(COLLECTION_ACCESS);

        let query = doc! { "id": id };
        let update = doc! { "access": bson::to_bson(&access).unwrap() };
        collection.update_one(query, update , None).await.map_err(mongo_error)?;
        Ok(())
    }

    pub async fn get_user_access(&self, id: String) -> Result<Access, Error> {
        let collection = self.db.collection(COLLECTION_ACCESS);

        let query = doc! { "id": id };
        if let Some(doc) = collection.find_one(query, None).await.map_err(mongo_error)? {
            if let Some(bson) = doc.get("access") {
                if let Ok(access) = bson::from_bson::<Access>(bson.clone()) {
                    return Ok(access);
                }
            }
        }
        Err(Error::NoRecord)
    }
}
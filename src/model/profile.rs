use std::iter::FilterMap;

use super::{Model, notify::NotifyProfile};
use super::error::*;

use mongodb::{
    bson,
    bson::{ doc, Bson, oid::ObjectId, },
    Cursor,
};
use serde::{Serialize, Deserialize};
use tokio::stream::StreamExt;


#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Access {
    Root,
    Admin,
    User,
}

#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub uid: String,
    pub description: String,
    pub secret: String,
    pub access: Access,
    pub services: Vec<ServiceRecord>,
}

pub const KEY_ID: &str = "uid";
pub const KEY_SERVICES: &str = "services";
pub const KEY_ACCESS: &str = "access";
pub const KEY_SECRET: &str = "secret";

#[derive(Serialize, Deserialize)]
pub enum Service {
    EmailNotify(super::notify::NotifyProfile),
    UserManagement(super::notify::AccessGrant),
    ModifyProfile(super::notify::ModifyProfile),
}

pub trait ExtractProfile<T> {
    fn extract_from(service: &Service) -> Option<&Self>;
}

#[derive(Serialize, Deserialize)]
pub struct ServiceRecord {
    pub _id: ObjectId,
    pub profile: Service,
}

impl ServiceRecord {
    pub fn new(service: Service) -> Self {
        Self {
            _id: ObjectId::new(),
            profile: service,
        }
    }
}

pub const COLLECTION_PROFILE: &str = "profile";

macro_rules! id_query {
    ($id: expr) => (doc! { KEY_ID: $id })
}

impl Model {
    pub async fn get_all_profile(&self) -> Result<Vec<Profile>, Error> {
        let collection = self.db.collection(COLLECTION_PROFILE);
        let cursor = collection.find(doc! {}, None).await.map_err(mongo_error)?;

        let user_list: Vec<Profile> = cursor.filter_map(|result| {
            if let Ok(doc) = result {
                if let Ok(profile) = bson::from_document::<Profile>(doc) {
                    Some(profile)
                } else {
                    None
                }
            } else {
                None
            }
        }).collect().await;

        Ok(user_list)
    }

    pub async fn add_profile(&self, profile: Profile) -> Result<ObjectId, Error> {
        let collection = self.db.collection(COLLECTION_PROFILE);
        let doc = bson::to_document(&profile).unwrap();

        let result = collection.insert_one(doc, None).await.map_err(mongo_error)?;
        Ok(result.inserted_id.as_object_id().unwrap().clone())
    }

    pub async fn get_profile(&self, id: String) -> Result<Profile, Error> {
        let coll =  self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let doc = coll.find_one(query, None)
            .await.map_err(mongo_error)?
            .ok_or(Error::NoRecord)?;
        Ok(bson::from_document(doc).map_err(bson_de_error)?)
    }

    pub async fn get_secret(&self, id: String) -> Result<String, Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let result = coll.find_one(query, None)
            .await.map_err(mongo_error)?
            .ok_or(Error::NoRecord)?
            .get_str(KEY_SECRET)
            .map_err(|_| Error::NoRecord)?
            .to_string();
        Ok(result)
    }

    pub async fn set_access(&self, id: String, access: Access) -> Result<(), Error> {
        let collection = self.db.collection(COLLECTION_PROFILE);

        let query = id_query!(id);
        let update = doc! { "access": bson::to_bson(&access).unwrap() };
        collection.update_one(query, update , None).await.map_err(mongo_error)?;
        Ok(())
    }

    pub async fn get_access(&self, id: String) -> Result<Access, Error> {
        let collection = self.db.collection(COLLECTION_PROFILE);

        let query = id_query!(id);
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
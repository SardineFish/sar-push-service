extern crate hex;
extern crate openssl;

use super::{Model};
use super::error::*;

use mongodb::{
    bson,
    bson::{ doc, Bson, oid::ObjectId, },
    Cursor,
};
use serde::{Serialize, Deserialize};
use tokio::stream::StreamExt;


#[derive(Serialize, Deserialize, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Access {
    Root = 4,
    Admin = 2,
    User = 0,
}

#[derive(Serialize, Deserialize)]
pub struct UserProfile {
    _id: ObjectId,
    pub uid: String,
    pub name: String,
    pub access: Access,
    pub description: String,
    pub secret: String,
    pub services: Vec<ServiceRecord>,
}

pub const KEY_ID: &str = "uid";
pub const KEY_SERVICES: &str = "services";
pub const KEY_SECRET: &str = "secret";

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content="profile")]
pub enum Service {
    UserAccessControl(AccessManagerProfile),
    EmailNotify(super::notify::NotifyProfile),
    ServiceManagement(super::service::ServiceManagerProfile),
}

pub trait ExtractProfile<T> {
    fn extract_from(service: &Service) -> Option<&Self>;
}

#[derive(Serialize, Deserialize)]
pub struct ServiceRecord {
    pub _id: ObjectId,
    pub service: Service,
}

impl ServiceRecord {
    pub fn new(service: Service) -> Self {
        Self {
            _id: ObjectId::new(),
            service: service,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AccessManagerProfile {
    pub access: Access,
}

impl ExtractProfile<AccessManagerProfile> for AccessManagerProfile {
    fn extract_from(service: &Service) -> Option<&Self> {
        match service {
            Service::UserAccessControl(profile) => Some(profile),
            _ => None,
        }
    }
}

pub const COLLECTION_PROFILE: &str = "profile";

macro_rules! id_query {
    ($id: expr) => (doc! { KEY_ID: $id })
}

impl Model {
    pub async fn get_all_profile(&self) -> Result<Vec<UserProfile>, Error> {
        let collection = self.db.collection(COLLECTION_PROFILE);
        let cursor = collection.find(doc! {}, None).await.map_err(mongo_error)?;

        let user_list: Vec<UserProfile> = cursor.filter_map(|result| {
            if let Ok(doc) = result {
                if let Ok(profile) = bson::from_document::<UserProfile>(doc) {
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

    pub fn gen_secret(&self) -> String {
        let mut secret: [u8; 32] = [0; 32];
        openssl::rand::rand_bytes(&mut secret).unwrap();
        openssl::base64::encode_block(&secret)
    }

    pub fn new_user(&self, name: String, description: String, access: Access) -> UserProfile {
        let oid = ObjectId::new();
        let uid = hex::encode(oid.bytes());
        let secret = self.gen_secret();
        UserProfile {
            _id: oid,
            uid: uid,
            secret: secret,
            name: name,
            description: description,
            access: access,
            services: vec![],
        }
    }

    pub async fn add_profile(&self, profile: UserProfile) -> Result<ObjectId, Error> {
        let collection = self.db.collection(COLLECTION_PROFILE);
        let doc = bson::to_document(&profile).unwrap();

        let result = collection.insert_one(doc, None).await.map_err(mongo_error)?;
        Ok(result.inserted_id.as_object_id().unwrap().clone())
    }

    pub async fn get_profile(&self, id: &str) -> Result<UserProfile, Error> {
        let coll =  self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let doc = coll.find_one(query, None)
            .await.map_err(mongo_error)?
            .ok_or(Error::NoRecord)?;
        Ok(bson::from_document(doc).map_err(bson_de_error)?)
    }

    pub async fn remove_user(&self, id: &str) -> Result<(), Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let result = coll.delete_one(query, None).await.map_err(mongo_error)?;
        Ok(())
    }

    pub async fn update_profile(&self, profile: UserProfile) -> Result<UserProfile, Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(&profile.uid);
        let update = doc! {
            "$set": bson::to_bson(&profile).unwrap()
        };
        let result = coll.update_one(query, update, None)
            .await.map_err(mongo_error)?;
        if result.modified_count <= 0 {
            Err(Error::NoRecord)
        } else {
            Ok(profile)
        }
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

    pub async fn revoke_secret(&self, id: &str) -> Result<String, Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let new_secret = self.gen_secret();
        let update = doc! {
            "$set": {
                "secret": &new_secret,
            }
        };
        let result = coll.update_one(query, update, None)
            .await.map_err(mongo_error)?;
        
        if result.matched_count <= 0 {
            Err(Error::NoRecord)
        } else {
            Ok(new_secret)
        }

    }

    // pub async fn set_access(&self, id: String, access: Access) -> Result<(), Error> {
    //     let collection = self.db.collection(COLLECTION_PROFILE);

    //     let query = id_query!(id);
    //     let update = doc! { "access": bson::to_bson(&access).unwrap() };
    //     collection.update_one(query, update , None).await.map_err(mongo_error)?;
    //     Ok(())
    // }

    // pub async fn get_access(&self, id: String) -> Result<Access, Error> {
    //     let collection = self.db.collection(COLLECTION_PROFILE);

    //     let query = id_query!(id);
    //     if let Some(doc) = collection.find_one(query, None).await.map_err(mongo_error)? {
    //         if let Some(bson) = doc.get("access") {
    //             if let Ok(access) = bson::from_bson::<Access>(bson.clone()) {
    //                 return Ok(access);
    //             }
    //         }
    //     }
    //     Err(Error::NoRecord)
    // }
}
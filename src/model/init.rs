use super::{
    Model,
    profile::COLLECTION_PROFILE,
};
use mongodb::{
    error::Error as MongoError,
    bson::{
        self,
        doc,
    }
};

impl Model {
    pub async fn init_db(&self) -> Result<(), MongoError> {
        let mut options = mongodb::options::CreateCollectionOptions::default();
        options.validation = Some(doc! {
            "bsonType": "object",
            "required": [ "uid", "description", "secret", "access" ],
            "properties": {
                "uid": {
                    "bsonType": "string",
                },
                "description": {
                    "bsonType": "string",
                },
                "secret": {
                    "bsonType": "string",
                },
                "access": {
                    "bsonType": "object",
                }
            }
        });
        
        self.db.create_collection(COLLECTION_PROFILE, options).await?;
        
        Ok(())
    }
}
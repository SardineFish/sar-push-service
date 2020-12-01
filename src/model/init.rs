use super::{Access, Error, Model, Profile, Service, ServiceRecord, notify::AccessGrant, profile::COLLECTION_PROFILE, error::mongo_error};
use mongodb::{
    error::Error as MongoError,
    bson::{
        self,
        doc,
    }
};
use log::{info, warn};

impl Model {
    pub async fn init_db(&self) -> Result<(), Error> {
        info!("Init database...");
        let mut options = mongodb::options::CreateCollectionOptions::default();
        // options.validation = Some(doc! {
        //     "bsonType": "object",
        //     "required": [ "uid", "description", "secret" ],
        //     "properties": {
        //         "uid": {
        //             "bsonType": "string",
        //         },
        //         "description": {
        //             "bsonType": "string",
        //         },
        //         "secret": {
        //             "bsonType": "string",
        //         },
        //     }
        // });
        
        info!("Create profile collection...");
        self.db.create_collection(COLLECTION_PROFILE, options)
            .await.map_err(mongo_error)?;
        
        info!("Init root user...");
        let root = Profile {
            uid: "root".to_string(),
            secret: "secret_must_change".to_string(),
            description: "Root user".to_string(),
            access: Access::Root,
            services: vec![
                ServiceRecord::new(Service::UserManagement(AccessGrant {}))
            ]
        };
        self.add_profile(root).await?;
        warn!("Root user with secret 'secret_must_change' must be change after init.");
        Ok(())
    }
}
use super::{
    access::COLLECTION_PROFILE, error::mongo_error, service, Access, AccessManagerProfile, Error,
    Model, Service, ServiceManagerProfile, ServiceRecord, UserProfile,
};
use log::{info, warn};
use mongodb::{
    bson::{self, doc},
    error::Error as MongoError,
};

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
        self.db
            .create_collection(COLLECTION_PROFILE, options)
            .await
            .map_err(mongo_error)?;

        info!("Init root user...");
        let root = UserProfile {
            uid: "root".to_string(),
            secret: "secret_must_change".to_string(),
            description: "Root user".to_string(),
            access: Access::Root,
            services: vec![
                // Root user can only used to create admin user.
                ServiceRecord::new(Service::UserAccessControl(AccessManagerProfile {
                    access: Access::Root,
                })),
                ServiceRecord::new(Service::ServiceManagement(ServiceManagerProfile {
                    access: Access::Root,
                })),
            ],
        };
        self.add_profile(root).await?;
        warn!("Root user with secret 'secret_must_change' must be change after init.");
        Ok(())
    }
}

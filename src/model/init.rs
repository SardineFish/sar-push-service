use super::{
    profile::COLLECTION_PROFILE,
    error::mongo_error, Access, AccessManagerProfile, Error,
    Model, Service, ServiceManagerProfile, ServiceRecord,
};
use log::{info, warn};

impl Model {
    pub async fn init_db(&self) -> Result<(), Error> {
        info!("Init database...");
        let options = mongodb::options::CreateCollectionOptions::default();
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

        self.remove_user("root").await?;
        let mut root = self.new_user("Root User".to_string(), "Root user.".to_string(), Access::Root);
        root.uid = "root".to_string();
        root.secret = "secret_must_change".to_string();
        root.services
            .push(ServiceRecord::new(Service::UserAccessControl(
                AccessManagerProfile {
                    access: Access::Root,
                },
            )));
        root.services
            .push(ServiceRecord::new(Service::ServiceManagement(
                ServiceManagerProfile {
                    access: Access::Root,
                },
            )));
        self.add_profile(root).await?;
        warn!("Root user with secret 'secret_must_change' must be change after init.");
        Ok(())
    }
}

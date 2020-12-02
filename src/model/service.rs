use std::vec;

use super::{
    Model,
    error::*,
    access::*
};

use mongodb::{
    bson::{
        self,
        doc,
        oid::ObjectId
    }
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ServiceManagerProfile {
    pub access: Access,
}


impl ExtractProfile<ServiceManagerProfile> for ServiceManagerProfile {
    fn extract_from(service: &Service) -> Option<&Self> {
        match service {
            Service::ServiceManagement(profile) => Some(profile),
            _ => None
        }
    }
}

macro_rules! id_query {
    ($id: expr) => (doc! { KEY_ID: $id })
}

impl Model {
    
    pub async fn get_services(&self, id: String) -> Result<Vec<ServiceRecord>, Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let service_docs: Vec<ServiceRecord> = coll.find_one(query, None)
            .await.map_err(mongo_error)?
            .ok_or_else(||Error::NoRecord)?
            .get_array(KEY_SERVICES).map_err(doc_error)?
            .into_iter()
            .filter_map(|val| {
                bson::from_bson::<ServiceRecord>(val.clone()).ok()
            })
            .collect();
        Ok(service_docs)
    }
    pub async fn add_service(&self, id: String, service: ServiceRecord) -> Result<(), Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let update = doc! {
            "$push": {
                KEY_SERVICES: bson::to_bson(&service).unwrap(),
            }
        };
        coll.update_one(query, update, None)
            .await.map_err(mongo_error)?;
        Ok(())
    }
    pub async fn remove_service(&self, id: String, service: ServiceRecord) -> Result<(), Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let update = doc! {
            "$pull": {
                KEY_SERVICES: {
                    "_id": service._id
                }
            }
        };
        coll.update_one(query, update, None).await.map_err(mongo_error)?;
        Ok(())
    }
    pub async fn update_service(&self, id: String, service: ServiceRecord) -> Result<(), Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let update = doc! {
            "$set": {
                "services.$[element].profile": bson::to_bson(&service.service).unwrap(),
            }
        };
        let mut option = mongodb::options::UpdateOptions::default();
        option.array_filters = Some(vec![
            doc! {
                "element._id": service._id,
            }
        ]);
        coll.update_one(query, update, Some(option)).await.map_err(mongo_error)?;
        Ok(())
    }
}
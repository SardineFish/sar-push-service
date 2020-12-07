use std::vec;

use super::{
    Model,
    error::*,
    profile::*,
};

use bson::oid::ObjectId;
use mongodb::{
    bson::{
        self,
        doc,
    }
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
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

impl ValidateProfile for ServiceManagerProfile {
    fn validate_properties(&self, profile: &super::service::ServiceManagerProfile) -> bool {
        self.access < profile.access
    }
}

macro_rules! id_query {
    ($id: expr) => (doc! { KEY_ID: $id })
}

impl Model {
    #[allow(dead_code)]
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
    pub async fn add_service(&self, id: &str, service: Service) -> Result<ServiceRecord, Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = doc! {
            "uid": id,
            "services": {
                "$not": {
                    "$elemMatch": {
                        "service.type": service.type_name()
                    }
                }
            }
        };
        let record = ServiceRecord::new(service);
        let update = doc! {
            "$push": {
                KEY_SERVICES: bson::to_bson(&record).unwrap(),
            }
        };
        let result = coll.update_one(query, update, None)
            .await.map_err(mongo_error)?;
        if result.matched_count <= 0 {
            Err(Error::NoRecord)
        } else {
            Ok(record)
        }
    }
    pub async fn remove_service(&self, uid: &str, service: ServiceRecord) -> Result<ServiceRecord, Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(uid);
        let update = doc! {
            "$pull": {
                KEY_SERVICES: {
                    "_id": service._id.clone()
                }
            }
        };
        coll.update_one(query, update, None).await.map_err(mongo_error)?;
        Ok(service)
    }
    pub async fn update_service(&self, id: &str, service: ServiceRecord) -> Result<(), Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = id_query!(id);
        let update = doc! {
            "$set": {
                "services.$[elem].service": bson::to_bson(&service.service).unwrap(),
            }
        };
        let mut option = mongodb::options::UpdateOptions::default();
        option.array_filters = Some(vec![
            doc! {
                "elem._id": service._id,
            }
        ]);
        let result = coll.update_one(query, update, Some(option)).await.map_err(mongo_error)?;
        if result.matched_count <= 0 {
            Err(Error::NoRecord)
        } else {
            Ok(())
        }
    }

    pub async fn get_service_by_id(&self, service_id: &ObjectId) -> Result<Service, Error> {
        let profile = self.get_service_owner(service_id).await?;
        let service = profile.services.into_iter()
            .find(|s| &s._id == service_id)
            .ok_or(Error::NoRecord)?;

        Ok(service.service)
    }

    pub async fn get_service_owner(&self, service_id: &ObjectId) -> Result<UserProfile, Error> {
        let coll = self.db.collection(COLLECTION_PROFILE);
        let query = doc! {
            "services._id": service_id.clone(),
        };
        // log::debug!("{:?}", service_id);
        let result = coll.find_one(query, None)
            .await
            .map_err(Error::from)?
            .ok_or(Error::NoRecord)?;
        
        let profile: UserProfile = bson::from_document(result).map_err(Error::from)?;

        Ok(profile)
    }
}
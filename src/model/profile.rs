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
    pub(super) _id: ObjectId,
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

pub const COLLECTION_PROFILE: &str = "profile";

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content="profile")]
pub enum Service {
    UserAccessControl(super::access::AccessManagerProfile),
    EmailNotify(super::notify::NotifyProfile),
    ServiceManagement(super::service::ServiceManagerProfile),
}

impl Service {
    pub fn type_name(&self) -> &'static str {
        match self {
            Service::UserAccessControl(_) => "UserAccessControl",
            Service::EmailNotify(_) => "EmailNotify",
            Service::ServiceManagement(_) => "ServiceManagement",
        }
    }
}

pub trait ValidateProfile {
    fn validate_properties(&self, profile: &super::service::ServiceManagerProfile) -> bool {
        profile.access >= Access::User
    }
}

impl ValidateProfile for Service {
    fn validate_properties(&self, manager_profile: &super::service::ServiceManagerProfile) -> bool {
        match self {
            Service::UserAccessControl(profile) => profile.validate_properties(manager_profile),
            Service::EmailNotify(profile) => profile.validate_properties(manager_profile),
            Service::ServiceManagement(profile) => profile.validate_properties(manager_profile)
        }
    }
}

pub trait ExtractProfile<T> {
    fn extract_from(service: &Service) -> Option<&Self>;
}

#[derive(Serialize, Deserialize, Clone)]
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
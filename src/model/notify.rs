use serde::{Serialize, Deserialize};

use super::{ExtractProfile, Service};

#[derive(Serialize, Deserialize)]
pub struct NotifyProfile {
    pub address: String,
    pub password: String,
    pub smtp_addr: String,
}

impl ExtractProfile<NotifyProfile> for NotifyProfile {
    fn extract_from(service: &Service) -> Option<&Self> {
        match service {
            Service::EmailNotify(profile) => Some(profile),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AccessGrant {

}

impl ExtractProfile<AccessGrant> for AccessGrant {
    fn extract_from(service: &Service) -> Option<&Self> {
        if let Service::UserManagement(profile) = service {
            Some(profile)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ModifyProfile {

}

impl ExtractProfile<ModifyProfile> for ModifyProfile {
    fn extract_from(service: &Service) -> Option<&Self> {
        match service {
            Service::ModifyProfile(profile) => Some(profile),
            _ => None
        }
    }
}
use serde::{Serialize, Deserialize};

use super::{ExtractProfile, Service};

#[derive(Serialize, Deserialize, Clone)]
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


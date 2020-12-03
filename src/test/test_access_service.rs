use actix_rt;
use actix_web::test::TestRequest;
use serde::{Serialize, Deserialize};
use crate::model::{self, Access, UserProfile};

use super::config_app;

#[derive(Serialize)]
struct UserAccessProfile {
    uid: String,
    secret: String,
}

#[derive(Serialize, Deserialize)]
struct PublicUserProfile {
    name: String,
    description: String,
    access: Access,
}

impl From<model::UserProfile> for PublicUserProfile {
    fn from(profile: UserProfile) -> Self {
        Self {
            name: profile.name,
            description: profile.description,
            access: profile.access,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct UserProfileWithUID {
    uid: String,
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct UserProfilePartial {
    name: Option<String>,
    description: Option<String>,
}

#[actix_rt::test]
async fn test_user_creation() {
    let app = config_app();
    let data = PublicUserProfile {
        name: "Test name".to_string(),
        description: "Test description.".to_string(),
        access: Access::Admin
    };
}

#[actix_rt::test]
async fn test_user_profile() {
    let app = config_app();
}
use actix_http::http::StatusCode;
use actix_rt;
use actix_web::{dev::ServiceResponse, test::TestRequest};
use actix_web::test;
use log::info;
use serde::{Serialize, Deserialize};
use crate::model::{self, Access, UserProfile};
use super::{TEST_ROOT_SECRET, helper::*};

use super::{TEST_ROOT_UID, config_app};

#[derive(Serialize, Deserialize)]
struct UserAccessProfile {
    uid: String,
    secret: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
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

// GET /access/user/{uid}
#[actix_rt::test]
async fn test_user_profile() {
    let mut app = config_app().await;

    TestRequest::get().uri(&format!("/access/user/{}", TEST_ROOT_UID))
        .auth(TEST_ROOT_UID, TEST_ROOT_SECRET)
        .send_request(&mut app).await
        .expect_status(StatusCode::OK)
        .into_json::<PublicUserProfile>().await;
}

// GET /access/user/{uid}
// POST /access/user
#[actix_rt::test]
async fn test_user_creation() {
    let mut app = config_app().await;
    let data = PublicUserProfile {
        name: "Test admin".to_string(),
        description: "A test admin's description.".to_string(),
        access: Access::Admin
    };

    // Add admin with Root should be ok
    let admin_one: UserAccessProfile = TestRequest::post().uri("/access/user")
        .auth(TEST_ROOT_UID, TEST_ROOT_SECRET)
        .set_json(&data)
        .send_request(&mut app).await
        .expect_status(StatusCode::OK)
        .into_json::<UserAccessProfile>().await;

    // Query lower user profile should be ok
    let profile: PublicUserProfile = TestRequest::get().uri(&format!("/access/user/{}", &admin_one.uid))
        .auth(TEST_ROOT_UID, TEST_ROOT_SECRET)
        .send_request(&mut app).await
        .expect_status(StatusCode::OK)
        .into_json::<PublicUserProfile>().await;
    assert_eq!(profile, data);

    // Query self profile should be ok
    TestRequest::get().uri(&format!("/access/user/{}", &admin_one.uid))
        .auth(&admin_one.uid, &admin_one.secret)
        .send_request(&mut app).await
        .expect_status(StatusCode::OK)
        .into_json::<PublicUserProfile>().await;

    // Query Root profile from Admin should be forbidden
    TestRequest::get().uri(&format!("/access/user/{}", TEST_ROOT_UID))
        .auth(&admin_one.uid, &admin_one.secret)
        .send_request(&mut app).await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data().await;

    
}

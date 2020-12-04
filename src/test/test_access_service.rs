use super::{helper::*, TEST_ROOT_SECRET};
use crate::model::{self, Access, UserProfile};
use actix_http::http::StatusCode;
use actix_rt;
use actix_web::test;
use actix_web::{dev::ServiceResponse, test::TestRequest};
use log::info;
use serde::{Deserialize, Serialize};

use super::{config_app, TEST_ROOT_UID};

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

    test_case("Unauthorized request should be block", async {
        TestRequest::get()
            .uri(&format!("/access/user/{}", TEST_ROOT_UID))
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error_data()
            .await;
    }).await;

    test_case("Authorized query of self profile should be ok", async {
        TestRequest::get()
            .uri(&format!("/access/user/{}", TEST_ROOT_UID))
            .auth(TEST_ROOT_UID, TEST_ROOT_SECRET)
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<PublicUserProfile>()
            .await;
    }).await;
    
}

// GET /access/user/{uid}
// POST /access/user
#[actix_rt::test]
async fn test_user_creation() {
    let mut app = config_app().await;
    let data = PublicUserProfile {
        name: "Test admin".to_string(),
        description: "A test admin's description.".to_string(),
        access: Access::Admin,
    };

    let admin_one: UserAccessProfile = test_case("Add Admin with Root should be ok", async {
        TestRequest::post()
            .uri("/access/user")
            .auth(TEST_ROOT_UID, TEST_ROOT_SECRET)
            .set_json(&data)
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserAccessProfile>()
            .await
    })
    .await;

    let profile = test_case("Query lower user profile should be ok", async {
        let profile: PublicUserProfile = TestRequest::get()
            .uri(&format!("/access/user/{}", &admin_one.uid))
            .auth(TEST_ROOT_UID, TEST_ROOT_SECRET)
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<PublicUserProfile>()
            .await;
        assert_eq!(profile, data);
        profile
    })
    .await;

    test_case("Query self profile should be ok", async {
        TestRequest::get()
            .uri(&format!("/access/user/{}", &admin_one.uid))
            .auth(&admin_one.uid, &admin_one.secret)
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<PublicUserProfile>()
            .await;
    })
    .await;

    test_case("Query Root profile from Admin should be forbidden", async {
        TestRequest::get()
            .uri(&format!("/access/user/{}", TEST_ROOT_UID))
            .auth(&admin_one.uid, &admin_one.secret)
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    })
    .await;
}

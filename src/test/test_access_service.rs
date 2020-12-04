use super::{AppType, TEST_ROOT_SECRET, helper::*};
use crate::{model::{self, Access, UserProfile}, test_case};
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

impl PublicUserProfile {
    pub fn new_for_test(access: Access) -> Self {
        Self {
            name: format!("Test user {:?}", access),
            description: format!("Test description {:?}", access),
            access: access
        }
    }
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

#[derive(Serialize, Deserialize, Default)]
struct UserProfilePartial {
    name: Option<String>,
    description: Option<String>,
    access: Option<Access>,
}

async fn request_add_user(app: &mut AppType, auth: &UserAccessProfile, profile: &PublicUserProfile) -> ServiceResponse {
    TestRequest::post()
        .uri("/access/user")
        .auth(&auth.uid, &auth.secret)
        .set_json(profile)
        .send_request(app)
        .await
}

async fn request_get_profile(app: &mut AppType, auth: &UserAccessProfile, uid: &str) -> ServiceResponse {
    TestRequest::get()
        .uri(&format!("/access/user/{}", uid))
        .auth(&auth.uid, &auth.secret)
        .send_request(app)
        .await
}

async fn request_update_profile(app: &mut AppType, auth: &UserAccessProfile, uid: &str, profile: &UserProfilePartial) -> ServiceResponse {
    TestRequest::patch()
        .uri(&format!("/access/user/{}", uid))
        .auth(&auth.uid, &auth.secret)
        .set_json(profile)
        .send_request(app)
        .await
}

async fn request_revoke_secret(app: &mut AppType, auth: &UserAccessProfile, uid: &str) -> ServiceResponse {
    TestRequest::post()
        .uri(&format!("/access/user/{}", uid))
        .auth(&auth.uid, &auth.secret)
        .send_request(app)
        .await
}

async fn request_delete_user(app: &mut AppType, auth: &UserAccessProfile, uid: &str) -> ServiceResponse {
    TestRequest::delete()
        .uri(&format!("/access/user/{}", uid))
        .auth(&auth.uid, &auth.secret)
        .send_request(app)
        .await
}

fn make_root_access() -> UserAccessProfile {
    UserAccessProfile {
        uid: TEST_ROOT_UID.to_string(),
        secret: TEST_ROOT_SECRET.to_string(),
    }
}

// GET /access/user/{uid}
#[actix_rt::test]
async fn test_user_profile() {
    let mut app = config_app().await;
    let root = make_root_access();

    test_case!("Unauthorized request should be block", async {
        TestRequest::get()
            .uri(&format!("/access/user/{}", TEST_ROOT_UID))
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error_data()
            .await;
    });

    test_case!("Authorized query of self profile should be ok", async {
        request_get_profile(&mut app, &root, TEST_ROOT_UID)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<PublicUserProfile>()
            .await;
    });
    
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
    let root = make_root_access();

    let admin: UserAccessProfile = test_case!("Add Admin with Root should be ok", async {
        request_add_user(&mut app, &root, &data)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserAccessProfile>()
            .await
    });

    
    test_case!("Add Root by Admin should be forbidden", async {
        request_add_user(&mut app, &admin, &PublicUserProfile::new_for_test(Access::Root))
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await
    });

    test_case!("Add Admin by Admin should be forbidden", async {
        request_add_user(&mut app, &admin, &PublicUserProfile::new_for_test(Access::Admin))
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await
    });

    let profile = test_case!("Query lower user profile should be ok", async {
        let profile: PublicUserProfile = request_get_profile(&mut app, &root, &admin.uid)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<PublicUserProfile>()
            .await;
        assert_eq!(profile, data);
        profile
    });

    test_case!("Query self profile should be ok", async {
        request_get_profile(&mut app, &admin, &admin.uid)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<PublicUserProfile>()
            .await;
    });

    test_case!("Query Root profile from Admin should be forbidden", async {
        request_get_profile(&mut app, &admin, &root.uid)
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("To cleanup, delete Admin by Root should be ok", async {
        request_delete_user(&mut app, &root, &admin.uid)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<PublicUserProfile>()
            .await;
        assert_eq!(profile, data);
    });
}

#[actix_rt::test]
async fn test_profile_update() {
    let mut app = config_app().await;
    let root = make_root_access();
    let mut admin_data = PublicUserProfile::new_for_test(Access::Admin);

    let admin: UserAccessProfile = test_case!("Add Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &admin_data)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserAccessProfile>()
            .await
    });

    let another_admin: UserAccessProfile = test_case!("Add another Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &admin_data)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserAccessProfile>()
            .await
    });

    admin_data.description = "Modified user description".to_string();
    let modified_user: PublicUserProfile = test_case!("Self modification be ok", async {
        let profile: PublicUserProfile = request_update_profile(&mut app, &admin, &admin.uid, &UserProfilePartial {
                description: Some(admin_data.description.clone()),
                ..Default::default()
            })
            .await
            .expect_status(StatusCode::OK)
            .into_json::<PublicUserProfile>()
            .await;
        assert_eq!(admin_data, profile);
        profile
    });

    test_case!("Upgrade self access should be forbidden", async {
        request_update_profile(&mut app, &admin, &admin.uid, &UserProfilePartial {
                access: Some(Access::Root),
                ..Default::default()
            })
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Modify profile with same access level should be forbidden", async {
        request_update_profile(&mut app, &admin, &another_admin.uid, &UserProfilePartial {
                description: Some(admin_data.description.clone()),
                ..Default::default()
            })
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Cleanup, delete Admin by Root should be ok", async {
        request_delete_user(&mut app, &root, &admin.uid)
            .await
            .expect_status(StatusCode::OK);
        request_delete_user(&mut app, &root, &another_admin.uid)
            .await
            .expect_status(StatusCode::OK);
    });
}


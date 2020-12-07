use super::{AppType, TEST_ROOT_SECRET, helper::*};
use crate::{model::{self, Access, UserProfile}, test_case};
use actix_http::http::StatusCode;
use actix_rt;
use actix_web::test;
use actix_web::{dev::ServiceResponse, test::TestRequest};
use log::info;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use super::{config_app, TEST_ROOT_UID};

#[derive(Serialize, Deserialize)]
pub struct UserAuth {
    pub uid: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UserInfo {
    pub name: String,
    pub description: String,
    pub access: Access,
}

impl UserInfo {
    pub fn new_for_test(access: Access) -> Self {
        Self {
            name: format!("Test user {:?}", access),
            description: format!("Test description {:?}", access),
            access: access
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
struct UserInfoPartial {
    name: Option<String>,
    description: Option<String>,
    access: Option<Access>,
}

async fn request_add_user(app: &mut AppType, auth: &UserAuth, profile: &UserInfo) -> ServiceResponse {
    TestRequest::post()
        .uri("/access/user")
        .auth(&auth.uid, &auth.secret)
        .set_json(profile)
        .send_request(app)
        .await
}

async fn request_get_profile(app: &mut AppType, auth: &UserAuth, uid: &str) -> ServiceResponse {
    TestRequest::get()
        .uri(&format!("/access/user/{}", uid))
        .auth(&auth.uid, &auth.secret)
        .send_request(app)
        .await
}

async fn request_update_profile(app: &mut AppType, auth: &UserAuth, uid: &str, profile: &UserInfoPartial) -> ServiceResponse {
    TestRequest::patch()
        .uri(&format!("/access/user/{}", uid))
        .auth(&auth.uid, &auth.secret)
        .set_json(profile)
        .send_request(app)
        .await
}

async fn request_revoke_secret(app: &mut AppType, auth: &UserAuth, uid: &str) -> ServiceResponse {
    TestRequest::post()
        .uri(&format!("/access/user/{}/secret", uid))
        .auth(&auth.uid, &auth.secret)
        .send_request(app)
        .await
}

async fn request_delete_user(app: &mut AppType, auth: &UserAuth, uid: &str) -> ServiceResponse {
    TestRequest::delete()
        .uri(&format!("/access/user/{}", uid))
        .auth(&auth.uid, &auth.secret)
        .send_request(app)
        .await
}

pub fn make_root_access() -> UserAuth {
    UserAuth {
        uid: TEST_ROOT_UID.to_string(),
        secret: TEST_ROOT_SECRET.to_string(),
    }
}

pub async fn cleanup(mut app: AppType, root: UserAuth, users: Vec<UserAuth>) {
    test_case!("Cleanup, delete users by Root should be ok", async {
        for user in &users {
            request_delete_user(&mut app, &root, &user.uid)
                .await
                .expect_status(StatusCode::OK);
        }
    });
}

pub async fn add_user(app: &mut AppType, root: &UserAuth, info: &UserInfo) -> UserAuth {
    let user_access: UserAuth = test_case!("Add users with Root should always be ok", async {
        request_add_user(app, root, info)
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await
    });
    user_access
}

pub fn non_exists_id() -> String {
    ObjectId::new().to_string()
}

// GET /access/user/{uid}
#[actix_rt::test]
async fn test_user_profile() {
    let mut app = config_app().await;
    let root = make_root_access();

    let admin: UserAuth = test_case!("Add Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin))
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await
    });

    let another_admin: UserAuth = test_case!("Add Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin))
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await
    });

    test_case!("Authorized query of self profile should be ok", async {
        request_get_profile(&mut app, &root, TEST_ROOT_UID)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserInfo>()
            .await;
    });

    test_case!("Query profile of same level should be forbidden", async {
        request_get_profile(&mut app, &admin, &another_admin.uid)
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Query higher level profile should be forbidden", async {
        request_get_profile(&mut app, &admin, &root.uid)
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Query non-exists user should be 404", async {
        request_get_profile(&mut app, &root, &non_exists_id())
            .await
            .expect_status(StatusCode::NOT_FOUND)
            .expect_error_data()
            .await;
    });

    cleanup(app, root, vec![admin, another_admin]).await;
    
}

// GET /access/user/{uid}
// POST /access/user
#[actix_rt::test]
async fn test_user_creation() {
    let mut app = config_app().await;
    let data = UserInfo {
        name: "Test admin".to_string(),
        description: "A test admin's description.".to_string(),
        access: Access::Admin,
    };
    let root = make_root_access();

    let admin: UserAuth = test_case!("Add Admin with Root should be ok", async {
        request_add_user(&mut app, &root, &data)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserAuth>()
            .await
    });

    
    test_case!("Add higher level user should be forbidden", async {
        request_add_user(&mut app, &admin, &UserInfo::new_for_test(Access::Root))
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await
    });

    test_case!("Add same level user should be forbidden", async {
        request_add_user(&mut app, &admin, &UserInfo::new_for_test(Access::Admin))
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await
    });

    test_case!("Add user with invalid body should be bad request", async {
        TestRequest::post()
            .uri("/access/user")
            .auth(&root.uid, &root.secret)
            .header("Content-Type", "application/json")
            .set_payload(r#"{
                "name": "test name",
                "non_description": "..."
            }"#)
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::BAD_REQUEST)
            .expect_error_data()
            .await;
    });

    cleanup(app, root, vec![admin]).await;
}

#[actix_rt::test]
async fn test_profile_update() {
    let mut app = config_app().await;
    let root = make_root_access();
    let mut admin_data = UserInfo::new_for_test(Access::Admin);

    let admin: UserAuth = test_case!("Add Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &admin_data)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserAuth>()
            .await
    });

    let another_admin: UserAuth = test_case!("Add another Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &admin_data)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserAuth>()
            .await
    });

    admin_data.description = "Modified user description".to_string();
    let modified_user: UserInfo = test_case!("Self modification be ok", async {
        let profile: UserInfo = request_update_profile(&mut app, &admin, &admin.uid, &UserInfoPartial {
                description: Some(admin_data.description.clone()),
                ..Default::default()
            })
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserInfo>()
            .await;
        assert_eq!(admin_data, profile);
        profile
    });

    test_case!("Upgrade self access should be forbidden", async {
        request_update_profile(&mut app, &admin, &admin.uid, &UserInfoPartial {
                access: Some(Access::Root),
                ..Default::default()
            })
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Modify profile with same access level should be forbidden", async {
        request_update_profile(&mut app, &admin, &another_admin.uid, &UserInfoPartial {
                description: Some(admin_data.description.clone()),
                ..Default::default()
            })
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Modify profile of non-exists user should be 404", async {
        request_update_profile(&mut app, &root, &non_exists_id(), &UserInfoPartial {
                description: Some(admin_data.description.clone()),
                ..Default::default()
            })
            .await
            .expect_status(StatusCode::NOT_FOUND)
            .expect_error_data()
            .await;
    });

    test_case!("Modify profile with other fields should be ok and affect nothing", async {
        let profile: UserInfo = TestRequest::patch()
            .uri(&format!("/access/user/{}", &admin.uid))
            .auth(&root.uid, &root.secret)
            .header("Content-Type", "application/json")
            .set_payload(r#"{
                "secret": "Attempt to update secret"
            }"#)
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await;
        assert_eq!(profile, admin_data);
    });

    
    cleanup(app, root, vec![admin, another_admin]).await;
}

#[actix_rt::test]
async fn test_secret_revoke()
{
    let mut app = config_app().await;
    let root = make_root_access();

    let mut admin: UserAuth = test_case!("Add Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin))
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await
    });

    let another_admin: UserAuth = test_case!("Add another Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin))
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await
    });

    admin.secret = test_case!("Revoke self secret should be ok", async {
        let profile: UserAuth = request_revoke_secret(&mut app, &admin, &admin.uid)
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await;
        assert_eq!(profile.uid, admin.uid);
        assert_ne!(profile.secret, admin.secret);
        profile.secret
    });

    test_case!("Revoke Root secret by Admin should be forbidden", async {
        request_revoke_secret(&mut app, &admin, &root.uid)
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Revoke Admin secret by Admin should be forbidden", async {
        request_revoke_secret(&mut app, &admin, &another_admin.uid)
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Revoke Admin secret by Root should be ok", async {
        let profile: UserAuth = request_revoke_secret(&mut app, &root, &another_admin.uid)
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await;
        assert_eq!(profile.uid, another_admin.uid);
        assert_ne!(profile.secret, another_admin.secret);
    });

    test_case!("Revoke secret of non-exists user should be 404", async {
        request_revoke_secret(&mut app, &root, &non_exists_id())
            .await
            .expect_status(StatusCode::NOT_FOUND)
            .expect_error_data()
            .await;
    });

    cleanup(app, root, vec![admin, another_admin]).await;
}

#[actix_rt::test]
async fn test_user_delete() {
    let mut app = config_app().await;
    let root = make_root_access();

    let mut admin: UserAuth = test_case!("Add Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin))
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await
    });

    let another_admin: UserAuth = test_case!("Add another Admin by Root should be ok", async {
        request_add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin))
            .await
            .expect_status(StatusCode::OK)
            .into_json()
            .await
    });

    test_case!("Delete higher level user should be forbidden", async {
        request_delete_user(&mut app, &admin, &root.uid)
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Delete same level user should be forbidden", async {
        request_delete_user(&mut app, &admin, &another_admin.uid)
            .await
            .expect_status(StatusCode::FORBIDDEN)
            .expect_error_data()
            .await;
    });

    test_case!("Delete self should be ok", async {
        request_delete_user(&mut app, &admin, &admin.uid)
            .await
            .expect_status(StatusCode::OK)
            .into_json::<UserInfo>()
            .await;
    });

    test_case!("Delete self repeatly should be unauthorized", async {
        request_delete_user(&mut app, &admin, &admin.uid)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error_data()
            .await;
    });

    test_case!("Repeat deletion should be ok with no content", async {
        request_delete_user(&mut app, &root, &admin.uid)
            .await
            .expect_status(StatusCode::NO_CONTENT)
            .expect_empty()
            .await;
    });

    cleanup(app, root, vec![another_admin]).await;
}

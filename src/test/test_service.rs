use actix_http::http::StatusCode;
use actix_rt;
use actix_web::{dev::ServiceResponse, test::TestRequest};
use model::{Access, AccessManagerProfile, ServiceManagerProfile};

use crate::{model, test_case};

use super::{AppType, config_app, test_access_service::add_user, test_access_service::{non_exists_id, UserAuth, UserInfo, cleanup, make_root_access}};
use super::helper::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ServiceProfile {
    service_id: String,
    service: model::Service,
}

async fn request_get_service(app: &mut AppType, auth: &UserAuth, uid: &str) -> ServiceResponse {
    TestRequest::get()
        .uri(&format!("/service/profile/{}", uid))
        .auth(&auth.uid, &auth.secret)
        .send_request(app)
        .await
}

pub async fn request_add_service(app: &mut AppType, auth: &UserAuth, uid: &str, service: &model::Service) -> ServiceResponse {
    TestRequest::post()
        .uri(&format!("/service/profile/{}", uid))
        .auth(&auth.uid, &auth.secret)
        .set_json(service)
        .send_request(app)
        .await
}

async fn request_update_service(app: &mut AppType, auth: &UserAuth, uid: &str, service_id: &str, service: &model::Service) -> ServiceResponse {
    TestRequest::patch()
        .uri(&format!("/service/profile/{}/{}", uid, service_id))
        .auth(&auth.uid, &auth.secret)
        .set_json(service)
        .send_request(app)
        .await
}

async fn request_delete_service(app: &mut AppType, auth: &UserAuth, uid: &str, service_id: &str) -> ServiceResponse {
    TestRequest::delete()
        .uri(&format!("/service/profile/{}/{}", uid, service_id))
        .auth(&auth.uid, &auth.secret)
        .send_request(app)
        .await
}
 
#[actix_rt::test]
async fn test_service_creation() {
    let mut app = config_app().await;
    let root = make_root_access();

    let admin = add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin)).await;
    let another_admin = add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin)).await;
    let user = add_user(&mut app, &root, &UserInfo::new_for_test(Access::User)).await;
    
    let profile: ServiceProfile = test_case!("Add service for lower level user should be ok", async {
        request_add_service(&mut app, &root, &admin.uid, &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::Admin
        }))
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await
    });

    test_case!("Add service without service of 'Service Management' should be forbidden", async {
        request_add_service(&mut app, &another_admin, &user.uid, &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::User
        }))
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("Add duplicated service for lower level user should be conflict", async {
        request_add_service(&mut app, &root, &admin.uid, &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::Admin
        }))
        .await
        .expect_status(StatusCode::CONFLICT)
        .expect_error_data()
        .await;
    });

    test_case!("Add service for same level user should be forbidden", async {
        request_add_service(&mut app, &admin, &another_admin.uid, &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::Admin
        }))
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("Add service for higher level user should be forbidden", async {
        request_add_service(&mut app, &admin, &root.uid, &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::Root
        }))
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("Add service for non-exists user should be 404", async {
        request_add_service(&mut app, &root, &non_exists_id(), &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::Root
        }))
        .await
        .expect_status(StatusCode::NOT_FOUND)
        .expect_error_data()
        .await;
    });

    test_case!("Add duplicated service profile should be conflict", async {
        request_add_service(&mut app, &root, &admin.uid, &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::Admin
        }))
        .await
        .expect_status(StatusCode::CONFLICT)
        .expect_error_data()
        .await;
    });

    test_case!("Add invalid service for user should be bad request", async {
        TestRequest::post()
        .uri(&format!("/service/profile/{}", &admin.uid))
        .auth(&root.uid, &root.secret)
        .header("Content-Type", "application/json")
        .set_payload(r#"{
            "type": "InvalidServiceType",
            "profile": [ "invalid profile" ]
        }"#)
        .send_request(&mut app)
        .await
        .expect_status(StatusCode::BAD_REQUEST)
        .expect_error_data()
        .await;
    });

    cleanup(app, root, vec![admin, another_admin, user]).await;
}

#[actix_rt::test]
async fn test_service_update()
{
    let mut app = config_app().await;
    let root = make_root_access();

    let admin = add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin)).await;
    let another_admin = add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin)).await;
    
    let mut profile: ServiceProfile = test_case!("Add service for lower level user should be ok", async {
        request_add_service(&mut app, &root, &admin.uid, &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::User
        }))
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await
    });

    profile.service = model::Service::ServiceManagement(ServiceManagerProfile {
        access: Access::Admin
    });
    test_case!("Update lower level user's service profile should be ok", async {
        let result: ServiceProfile = request_update_service(&mut app, &root, &admin.uid, &profile.service_id, &profile.service)
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await;
        assert_eq!(result.service_id, profile.service_id);
        assert_eq!(result.service, profile.service);
    });

    test_case!("Update higher level user's service profile should be forbidden", async {
        request_update_service(&mut app, &admin, &root.uid, &profile.service_id, &profile.service)
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("Update same level user's service profile should be forbidden", async {
        request_update_service(&mut app, &admin, &another_admin.uid, &profile.service_id, &profile.service)
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });
    
    profile.service = model::Service::ServiceManagement(ServiceManagerProfile {
        access: Access::User
    });
    test_case!("Update self service profile should be ok", async {
        let result: ServiceProfile = request_update_service(&mut app, &admin, &admin.uid, &profile.service_id, &profile.service)
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await;
        assert_eq!(result.service_id, profile.service_id);
        assert_eq!(result.service, profile.service);
    });

    test_case!("Update non-exists user's service profile should be 404", async {
        request_update_service(&mut app, &root, &non_exists_id(), &profile.service_id, &profile.service)
        .await
        .expect_status(StatusCode::NOT_FOUND)
        .expect_error_data()
        .await;
    });

    test_case!("Update service profile not belong to specifc user should be 404", async {
        request_update_service(&mut app, &root, &another_admin.uid, &profile.service_id, &profile.service)
        .await
        .expect_status(StatusCode::NOT_FOUND)
        .expect_error_data()
        .await;
    });

    test_case!("Update non-exists service profile should be 404", async {
        request_update_service(&mut app, &root, &admin.uid, &non_exists_id(), &profile.service)
        .await
        .expect_status(StatusCode::NOT_FOUND)
        .expect_error_data()
        .await;
    });

    test_case!("Change of service type should be bad request", async {
        request_update_service(&mut app, &root, &admin.uid, &profile.service_id, &model::Service::UserAccessControl(AccessManagerProfile {
            access: Access::User,
        }))
        .await
        .expect_status(StatusCode::BAD_REQUEST)
        .expect_error_data()
        .await;
    });

    test_case!("Update service with invalid profile format should be bad request", async {
        TestRequest::patch()
        .uri(&format!("/service/profile/{}/{}", &admin.uid, &profile.service_id))
        .auth(&root.uid, &root.secret)
        .header("Content-Type", "application/json")
        .set_payload(r#"{
            "type": "ServiceManagement",
            "profile: {
                "invalid_field": [ "invalid value" ]
            }
        }"#)
        .send_request(&mut app)
        .await
        .expect_status(StatusCode::BAD_REQUEST)
        .expect_error_data()
        .await;
    });

    cleanup(app, root, vec![admin, another_admin]).await;
}

#[actix_rt::test]
async fn test_service_delete() {
    let mut app = config_app().await;
    let root = make_root_access();

    let admin = add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin)).await;
    let another_admin = add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin)).await;
    
    let profile: ServiceProfile = test_case!("Add service for lower level user should be ok", async {
        request_add_service(&mut app, &root, &admin.uid, &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::Admin
        }))
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await
    });

    let another_profile: ServiceProfile = test_case!("Add service for lower level user should be ok", async {
        request_add_service(&mut app, &root, &another_admin.uid, &model::Service::ServiceManagement(ServiceManagerProfile {
            access: Access::User
        }))
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await
    });

    test_case!("Delete service by same level manager should be forbidden", async {
        request_delete_service(&mut app, &admin, &another_admin.uid, &another_profile.service_id)
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("Delete service by lower level manager should be forbidden", async {
        request_delete_service(&mut app, &another_admin, &admin.uid, &profile.service_id)
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("Delete service not belongs to specific user should be ok with no content", async {
        request_delete_service(&mut app, &root, &admin.uid, &another_profile.service_id)
        .await
        .expect_status(StatusCode::NO_CONTENT)
        .expect_empty()
        .await;
    });

    test_case!("Delete self service should be ok", async {
        let result: ServiceProfile = request_delete_service(&mut app, &another_admin, &another_admin.uid, &another_profile.service_id)
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await;
        
        assert_eq!(result.service_id, another_profile.service_id);
        assert_eq!(result.service, another_profile.service);
    });

    test_case!("Delete service without service profile of 'Service Management' should be forbidden", async {
        request_delete_service(&mut app, &another_admin, &another_admin.uid, &another_profile.service_id)
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("Delete service by higher level manager should be ok", async {
        let result: ServiceProfile = request_delete_service(&mut app, &root, &admin.uid, &profile.service_id)
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await;
        
        assert_eq!(result.service_id, profile.service_id);
        assert_eq!(result.service, profile.service);
    });

    test_case!("Repeat deletion should be ok with no content", async {
        request_delete_service(&mut app, &root, &admin.uid, &profile.service_id)
        .await
        .expect_status(StatusCode::NO_CONTENT)
        .expect_empty()
        .await;
    });

    test_case!("Delete service profile of non-exists user should be 404", async {
        request_delete_service(&mut app, &root, &non_exists_id(), &profile.service_id)
        .await
        .expect_status(StatusCode::NOT_FOUND)
        .expect_error_data()
        .await;
    });

    cleanup(app, root, vec![admin, another_admin]).await;
}
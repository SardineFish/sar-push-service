use std::time::Duration;

use actix_http::{http::StatusCode};
use actix_rt;
use actix_web::{dev::ServiceResponse, test::TestRequest};

use crate::{model::{Access, NotifyProfile, Service}, test_case};

use super::{AppType, config_app, test_access_service::UserInfo, test_access_service::{UserAuth, add_user, cleanup, make_root_access, non_exists_id}, test_service::request_add_service};
use serde::{Serialize, Deserialize};
use super::helper::*;

#[derive(Deserialize, Serialize, Clone)]
struct NotifyRequest {
    to: String,
    subject: String,
    content_type: String,
    body: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
enum NotifyStatus {
    Pending,
    Sent,
    Error,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct PubNotifyInfo {
    message_id: String,
    status: NotifyStatus,
    error: Option<String>,
}

async fn send_notification(app: &mut AppType, root: &UserAuth, request: NotifyRequest) -> ServiceResponse {
    TestRequest::post()
        .uri("/notify/queue")
        .auth(&root.uid, &root.secret)
        .set_json(&request)
        .send_request(app)
        .await
}

async fn list_all_notifications(app: &mut AppType, auth: &UserAuth, uid: &str, filter: &str) -> ServiceResponse {
    TestRequest::get()
        .uri(&format!("/notify/all/{}?filter={}", uid, filter))
        .auth(&auth.uid, &auth.secret)
        .send_request(app)
        .await
}

async fn query_notification(app: &mut AppType, root: &UserAuth, message_id: &str) -> ServiceResponse {
    TestRequest::get()
        .uri(&format!("/notify/{}", message_id))
        .auth(&root.uid, &root.secret)
        .send_request(app)
        .await
}

#[actix_rt::test]
async fn test_notify() {
    let mut app = config_app().await;
    let root = make_root_access();

    let admin = add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin)).await;
    let another_admin = add_user(&mut app, &root, &UserInfo::new_for_test(Access::Admin)).await;

    let notify_request = NotifyRequest {
        to: "test@sardinefish.com".to_string(),
        subject: "Test Notification".to_string(),
        content_type: "text/plain".to_string(),
        body: "The text body of an email notification.".to_string(),
    };
    test_case!("Send notification without service profile should be forbidden", async {
        send_notification(&mut app, &root, notify_request.clone())
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("Add service profile to lower level user should be ok", async {
        request_add_service(&mut app, &root, &admin.uid, &Service::EmailNotify(NotifyProfile {
            smtp_address: "192.0.2.1".to_string(),
            tls: false,
            name: "Display Name".to_string(),
            username: "user@example.com".to_string(),
            password: "password".to_string(),
            email_address: "user@example.com".to_string(),
        }))
        .await
        .expect_status(StatusCode::OK);

        request_add_service(&mut app, &root, &another_admin.uid, &Service::EmailNotify(NotifyProfile {
            smtp_address: "192.0.2.1".to_string(),
            tls: false,
            name: "Display Name".to_string(),
            username: "user@example.com".to_string(),
            password: "password".to_string(),
            email_address: "user@example.com".to_string(),
        }))
        .await
        .expect_status(StatusCode::OK);
    });

    let mut notify: PubNotifyInfo = test_case!("Send notification should be ok", async {
        let result: PubNotifyInfo = send_notification(&mut app, &admin, notify_request.clone())
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await;
        assert_eq!(result.status, NotifyStatus::Pending);
        result
    });

    test_case!("Query non-exists notification should be 404", async {
        query_notification(&mut app, &admin, &non_exists_id())
        .await
        .expect_status(StatusCode::NOT_FOUND)
        .expect_error_data()
        .await;
    });

    // Wait for SMTP timtout
    actix_rt::time::delay_for(Duration::from_millis(600)).await;

    notify.status = NotifyStatus::Error;
    notify.error = Some("Cannot connect to SMTP Server".to_string());
    test_case!("Query previous sent notification should be ok with error status", async {
        let result: PubNotifyInfo = query_notification(&mut app, &admin, &notify.message_id)
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await;
        assert_eq!(result, notify); 
    });

    let mut another_notify = test_case!("Send notification again should be ok", async {
        let result: PubNotifyInfo = send_notification(&mut app, &admin, notify_request.clone())
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await;
        assert_eq!(result.status, NotifyStatus::Pending);
        result
    });

    test_case!("List all pending notification should be ok", async {
        let result: Vec<PubNotifyInfo> = list_all_notifications(&mut app, &admin, &admin.uid, "Pending")
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await;

        assert_eq!(result, vec![another_notify.clone()]);
    });

    // Wait for SMTP timtout
    actix_rt::time::delay_for(Duration::from_millis(600)).await;

    another_notify.status = NotifyStatus::Error;
    another_notify.error = Some("Cannot connect to SMTP Server".to_string());
    test_case!("List all notification should be ok", async {
        let result: Vec<PubNotifyInfo> = list_all_notifications(&mut app, &admin, &admin.uid, "All")
        .await
        .expect_status(StatusCode::OK)
        .into_json()
        .await;

        assert_eq!(result, vec![notify.clone(), another_notify.clone()]);
    });

    test_case!("Query other's notification should be forbidden", async {
        query_notification(&mut app, &another_admin, &notify.message_id)
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("List other's notification should be forbidden", async {
        list_all_notifications(&mut app, &another_admin, &admin.uid, "All")
        .await
        .expect_status(StatusCode::FORBIDDEN)
        .expect_error_data()
        .await;
    });

    test_case!("Send invalid notification should be bad request", async {
        TestRequest::post()
        .uri("/notify/queue")
        .auth(&admin.uid, &admin.secret)
        .header("Content-Type", "application/json")
        .set_payload(r#"{
            "from": "<Unknown user>",
            "cc": "cc@example.com",
            "content_type": "plain/text",
        }"#)
        .send_request(&mut app)
        .await
        .expect_status(StatusCode::BAD_REQUEST)
        .expect_error_data()
        .await;
    });

    cleanup(app, root, vec![admin, another_admin]).await;
}
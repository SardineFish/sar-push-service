use actix_http::http::StatusCode;
use actix_rt;
use actix_web::test::{TestRequest};
use crate::test_case;

use super::{config_app, helper::*, test_access_service::make_root_access};

#[actix_rt::test]
async fn test_auth() {
    let mut app = config_app().await;
    let root = make_root_access();

    test_case!("Empty auth request should be unauthorized", async {
        TestRequest::get()
            .uri(&format!("/access/user/{}", &root.uid))
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error_data()
            .await;
    });

    test_case!("Auth with non-exists user should be unauthorized", async {
        TestRequest::get()
            .uri(&format!("/access/user/{}", &root.uid))
            .auth("This uid must not exists", "password")
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error_data()
            .await;
    });

    test_case!("Auth with incorrect secret should be unauthorized", async {
        TestRequest::get()
            .uri(&format!("/access/user/{}", &root.uid))
            .auth(&root.uid, "incorrect password")
            .send_request(&mut app)
            .await
            .expect_status(StatusCode::UNAUTHORIZED)
            .expect_error_data()
            .await;
    });
}
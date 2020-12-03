mod helper;
mod test_access_service;

use actix_web::{App, web::Json, dev::{MessageBody, Service, ServiceRequest, ServiceResponse}, test};
use actix_http::Request;
use futures::executor::block_on;
use mongodb::bson::oid::ObjectId;
use std::{time::Duration, thread::spawn};
use actix_rt::time;

use crate::model::{Model, UserProfile};

const TEST_ADDR: &str = "localhost:3000";
const TEST_ROOT_UID: &str = "test-root";
const TEST_ROOT_SECRET: &str = "TEST_SECRET";


type AppType = impl Service<Request = Request, Response= ServiceResponse, Error = actix_web::Error>;

async fn config_app() -> AppType {
    
    test::init_service(
    App::new()
    ).await
}


#[actix_rt::test]
async fn init() {
    let model = Model::new().await.unwrap();
    let mut profile = model.new_user("Test Root".to_string(), "Root use for test".to_string(), crate::model::Access::Root);
    profile.uid = TEST_ROOT_UID.to_string();
    profile.secret = TEST_ROOT_SECRET.to_string();
    let oid = ObjectId::with_string("112233445566778899aabbcc").unwrap();
    profile.set_id(oid);
    model.add_profile(profile).await;
}

#[actix_rt::test]
async fn test_service_setup() {
    let server = super::start_server(TEST_ADDR).await.unwrap();

    let srv = server.clone();
    let thread = spawn(move || {
        block_on(async move {
            srv.await.unwrap();
        });
    });

    time::delay_for(Duration::from_millis(500)).await;

    server.stop(true).await;

    thread.join().unwrap();
}

#[actix_rt::test]
async fn test_access_service() {
    let mut app = config_app().await;
    let req = test::TestRequest::get().uri("/access/account").to_request();
    let response: ServiceResponse = test::call_service(&mut app, req).await;
    // let server = super::start_server(TEST_ADDR).await.unwrap();
    // test::init_service(app)
}
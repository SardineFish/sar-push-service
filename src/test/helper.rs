extern crate serde_json;
extern crate colored;

use actix_web::{App, web::Json, dev::{MessageBody, Service, ServiceRequest, ServiceResponse}, test};
use actix_http::{Request, http::StatusCode};
use futures::{Future, executor::block_on};
use test::TestRequest;
use std::{time::Duration, thread::spawn};
use actix_rt::time;
use std::pin::Pin;
use super::AppType;
use serde::{Serialize, Deserialize};
use colored::*;

use crate::utils::FutureRtnT;

pub trait TestRequestHelper {
    fn auth(self, username: &str, password: &str) -> Self;
}

impl TestRequestHelper for TestRequest {
    fn auth(self, username: &str, password: &str) -> Self {
        let auth = format!("{}:{}", username, password);
        let b64 = openssl::base64::encode_block(auth.as_bytes());
        self.header("Authorization", format!("Basic {}", b64))
    }
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    error: String,
}

pub trait TestResponseHelper {
    fn expect_status(self, code: StatusCode) -> Self;
    fn into_json<T: serde::de::DeserializeOwned>(self) -> Pin<Box<Future<Output = T>>>;
    fn expect_error_data(self) -> Pin<Box<Future<Output = ()>>>;
}

impl TestResponseHelper for ServiceResponse {
    fn expect_status(self, code: StatusCode) -> Self {
        assert_eq!(self.status(), code, "Expect status code {:?} found {:?}", code, self.status());
        self
    }
    fn into_json<T: serde::de::DeserializeOwned>(mut self) -> Pin<Box<Future<Output = T>>>  {
        Box::pin(async move {
            let body = test::read_body(self).await;

            serde_json::from_slice(&body).expect("Failed to deserialize json")

            // test::read_body_json(self).await
        })
    }
    fn expect_error_data(self) -> Pin<Box<Future<Output = ()>>> {
        Box::pin(async move {
            let body = test::read_body(self).await;

            serde_json::from_slice::<ErrorBody>(&body).expect(format!("Invalid body format {:?}", &body).as_str());
        })
    }
}

pub fn init_log() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
}

// pub async fn test_case<T>(description: &str, f: impl Future<Output=T>) -> T {
//     print!("Test '{}' ... ", description);
//     let result = f.await;
//     println!("{}", "pass".green());
//     result
// }

#[macro_export]
macro_rules! test_case {
    ($description: expr, $body: expr) => ({
        print!("Test '{}' at {}:{}:{} ... ", $description, file!(), line!(), column!());
        let result = $body.await;
        println!("{}", colored::Colorize::green("pass"));
        result
    })
}
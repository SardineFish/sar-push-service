use std::future::Future;

use reqwest::{RequestBuilder, Response};
use serde::{Serialize, Deserialize};
use std::pin::Pin;

use crate::auth::UserAuth;

pub trait RequestHelper {
    fn auth(self, auth: Option<UserAuth>) -> Self;
}

impl RequestHelper for RequestBuilder {
    fn auth(self, auth: Option<UserAuth>) -> Self {
        match auth {
            Some(auth) => self.basic_auth(auth.uid, Some(auth.secret)),
            _ => self
        }
        
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorReport {
    pub error: String,
}

pub trait ResponseHelper {
    fn handle_error(self) -> Pin<Box<dyn Future<Output=Self>>>;
}

impl ResponseHelper for Response {
    fn handle_error(self) -> Pin<Box<dyn Future<Output=Self>>> {
        Box::pin(async move {
            if self.status().is_client_error() || self.status().is_server_error() {
                let status = self.status();
                let err: ErrorReport = self.json().await.expect("Invalid response");
                panic!("Error-{}:{}", status, err.error);
            }

            self
        })
    }
}

pub fn output<T: Serialize>(data: T, output: Option<&str>) {
    if let Some(file) = output {
        std::fs::write(file, serde_json::to_string(&data).unwrap()).unwrap();
    } else {
        println!("{}", serde_yaml::to_string(&data).unwrap());
    }
}
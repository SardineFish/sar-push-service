use std::future::Future;

use reqwest::{RequestBuilder, Response};
use serde::{Serialize, Deserialize};
use std::pin::Pin;

use crate::{auth::UserAuth, error::{Error, Result}};

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
    type OutputSelf;
    fn handle_error(self) -> Pin<Box<dyn Future<Output=Self::OutputSelf>>>;
}

impl ResponseHelper for Response {
    type OutputSelf = Result<Self>;
    fn handle_error(self) -> Pin<Box<dyn Future<Output=Self::OutputSelf>>> {
        Box::pin(async move {
            if self.status().is_client_error() || self.status().is_server_error() {
                let status = self.status();
                let err: ErrorReport = self.json().await.map_err(Error::from)?;
                Err(Error::ResponseError(status, err.error))
            } else {
                Ok(self)
            }
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
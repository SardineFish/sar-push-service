use actix_web::{App, web::Json, dev::{MessageBody, Service, ServiceRequest, ServiceResponse}, test};
use actix_http::{Request, http::StatusCode};
use futures::{Future, executor::block_on};
use std::{time::Duration, thread::spawn};
use actix_rt::time;
use std::pin::Pin;
use super::AppType;

use crate::utils::FutureRtnT;



trait TestRequestHelper {
    fn call_service<'a>(self, app: &'a mut AppType) -> Pin<Box<Future<Output = ServiceResponse> + 'a >>;
}

impl TestRequestHelper for Request {
    fn call_service<'a>(self, app: &'a mut AppType) -> Pin<Box<Future<Output = ServiceResponse> + 'a >>  {
        Box::pin(async move {
            test::call_service(app, self).await
        })
    }
}

trait TestResponseHelper {
    fn expect_status(&self, code: StatusCode) -> &Self;
    fn into_json<'a, T: serde::de::DeserializeOwned>(self) -> FutureRtnT<'a, T> ; 
}

impl TestResponseHelper for ServiceResponse {
    fn expect_status(&self, code: StatusCode) -> &Self {
        assert_eq!(self.status(), code);
        self
    }
    fn into_json<'a, T: serde::de::DeserializeOwned>(self) -> FutureRtnT<'a, T> {
        Box::pin(async move {
            test::read_body_json(self).await
        })
    }
}
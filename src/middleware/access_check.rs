use actix_web::{web, HttpResponse, dev::{MessageBody, Service, ServiceRequest, ServiceResponse}};
use crate::{model::{Access, Model}};
use std::pin::Pin;
use std::fmt;
use serde::{Deserialize};
use futures::{
    Future,
};
use futures_util::FutureExt;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

#[derive(Deserialize)]
struct AccessQuery {
    pub id: String,
    pub secret: String,
}

#[derive(Debug)]
enum Error {
    InternalError(actix_web::Error),
    AccessDeny,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InternalError(_) => write!(f, "Internal Error"),
            Error::AccessDeny => write!(f, "Access denied")
        }
    }
}


pub async fn access_chk<S, B>(request: ServiceRequest, mut service: Rc<RefCell<S>>) -> Result<ServiceResponse<B>, actix_web::Error>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody
{
    let response = service.call(request).await.unwrap();
    Ok(response)
}


pub fn access_check<S, B>(at_least: Access) 
    -> fn(req: ServiceRequest, srv: Rc<RefCell<S>>) 
        -> Pin<Box<Future<Output = Result<ServiceResponse<B>, actix_web::Error>>>>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: MessageBody
{
    let f: fn(req: ServiceRequest, srv: Rc<RefCell<S>>) 
        -> Pin<Box<Future<Output = Result<ServiceResponse<B>, actix_web::Error>>>> 
        = |request, service| {
            Box::pin(async move {
                    let response = { service.borrow_mut().call(request)};
                    let response = response.await?;
                    Ok(response)
                })
        };
    f
}
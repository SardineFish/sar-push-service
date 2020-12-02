use actix_web::{web, HttpResponse, error as web_errors, dev::{MessageBody, Service, ServiceRequest, ServiceResponse}};
use model::Profile;
use crate::{model::{self, Access, Model, Error as ModelError}};
use std::pin::Pin;
use std::fmt;
use serde::{Deserialize};
use futures::{
    Future,
};
use futures_util::FutureExt;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use super::{func_middleware::*};
use actix_web_httpauth::extractors::{
    AuthExtractor,
    basic::BasicAuth,
};

#[derive(Deserialize)]
struct AccessQuery {
    pub id: String,
    pub secret: String,
}

#[derive(Debug)]
enum Error {
    InternalError(ModelError),
    AccessDeny,
    UnexpectedError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InternalError(_) => write!(f, "Internal Error"),
            Error::AccessDeny => write!(f, "Access denied"),
            Error::UnexpectedError => write!(f, "Unexpected internal error.")
        }
    }
}

fn map_error(err: ModelError) -> actix_web::Error {
    match err {
        ModelError::NoRecord => web_errors::ErrorForbidden(Error::AccessDeny),
        err => web_errors::ErrorInternalServerError(Error::InternalError(err)),
    }
}

async fn get_profile(request: &ServiceRequest) -> Result<model::Profile, actix_web::Error> {
    let model = request.app_data::<web::Data<Model>>().unwrap();
    let auth = BasicAuth::from_service_request(&request).await?;
    let id = auth.user_id().to_string();
    let password = auth.password()
        .ok_or(web_errors::ErrorUnauthorized("Unauthorized"))?;
    let profile = model.get_profile(id)
        .await.map_err(map_error)?;
    if &profile.secret != password {
        Err(web_errors::ErrorUnauthorized("Unauthorized"))
    } else {
        Ok(profile)
    }
}

pub async fn access_chk<S, B>(request: ServiceRequest, mut service: Rc<RefCell<S>>) -> Result<ServiceResponse<B>, actix_web::Error>
where
    S: ServiceT<B> + 'static,
    S::Future: 'static,
    B: MessageBody
{
    let profile = get_profile(&request).await;
    match profile {
        Ok(profile) => {
            let (req, payload) = request.into_parts();
            req.extensions_mut().insert(profile);
            let request = ServiceRequest::from_parts(req, payload)
                .map_err(|_| web_errors::ErrorInternalServerError(Error::UnexpectedError))?;
            let response = service.call(request).await.unwrap();
            Ok(response)
        },
        Err(err) => Ok(request.error_response(err))
    }
}

async_middleware!(pub authentication, access_chk);


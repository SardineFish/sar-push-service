use std::marker::PhantomData;
use std::vec::Vec;
use actix_web::{HttpRequest, dev::{MessageBody, RequestHead, ServiceRequest, ServiceResponse}, error as web_errors, guard::Guard};
use log::info;
use std::rc::Rc;
use std::cell::RefCell;

use super::func_middleware::*;
use crate::{
    model::{
        self,
        UserProfile, Service, ExtractProfile
    }
};

pub struct ServiceGuard<T>{
    phantom: PhantomData<T>,
}

impl<T> ServiceGuard<T> {
    pub fn new() -> ServiceGuard<T> {
        Self {
            phantom: PhantomData
        }
    }
}

async fn service_guard_async<S, B, T>(request: ServiceRequest, srv: Rc<RefCell<S>>) -> Result<ServiceResponse<B>, actix_web::Error>
where
    S: ServiceT<B>,
    S::Future: 'static,
    B: MessageBody,
    T: ExtractProfile<T> + Clone + 'static
{
    let (mut request, payload) = request.into_parts();

    let result = ( |request: &mut HttpRequest | -> Result<(), actix_web::Error> {
        let extentions = request.extensions();
        let user_profile = extentions.get::<UserProfile>()
            .ok_or(web_errors::ErrorInternalServerError("No profile found."))?;
        let service_profile = (&user_profile.services).into_iter()
            .find_map(|s| T::extract_from(&s.service));
        
        if let Some(profile) = service_profile {
            let profile = profile.clone();
            drop(user_profile);
            drop(extentions);
            request.extensions_mut().insert(profile);
            Ok(())
        } else {
            Err(web_errors::ErrorForbidden("Access denied"))
        }
    })(&mut request);

    let request = ServiceRequest::from_parts(request, payload)
        .map_err(|_| web_errors::ErrorInternalServerError("Unexpected Error"))?;

    match result {
        Ok(()) => {
            Ok(srv.borrow_mut().call(request).await?)
        },
        Err(err) => {
            Ok(request.error_response(err))
        }
    }
}

pub fn service_guard<T, S, B>() -> super::FuncMiddleware<S, AsyncMiddlewareRtn<B>>
where
    S: ServiceT<B> + 'static,
    S::Future: 'static,
    B: MessageBody,
    T: ExtractProfile<T> + Clone + 'static
{
    super::FuncMiddleware::from_func(move |req, srv| {
        Box::pin(async move {
            service_guard_async::<S, B, T>(req, srv).await
        })
    })
}

impl<T> Guard for ServiceGuard<T> where T : ExtractProfile<T> + Clone + 'static
{
    fn check(&self, request: &RequestHead) -> bool {
        (move|| -> Result<bool, actix_web::Error> {
            let extentions = request.extensions_mut();
            let user_profile = extentions.get::<UserProfile>()
                .ok_or(web_errors::ErrorInternalServerError("No profile found."))?;
            let service_profile = (&user_profile.services).into_iter()
                .find_map(|s| T::extract_from(&s.service));
            
            if let Some(profile) = service_profile {
                let profile = profile.clone();
                drop(user_profile);
                drop(extentions);
                request.extensions_mut().insert(profile);
                Ok(true)
            } else {
                Ok(false)
            }
        })().unwrap_or(false)
    }
}
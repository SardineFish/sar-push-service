use std::marker::PhantomData;
use std::vec::Vec;
use actix_web::{
    guard::Guard,
    dev::RequestHead,
    error as web_errors,
};

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

impl<T> Guard for ServiceGuard<T> where T : ExtractProfile<T> + Clone + 'static
{
    fn check(&self, request: &RequestHead) -> bool {
        (move|| -> Result<bool, actix_web::Error> {
            let extentions = request.extensions();
            let profile = extentions.get::<UserProfile>()
                .ok_or(web_errors::ErrorInternalServerError("No profile found."))?;
            let profile = (&profile.services).into_iter()
                .find_map(|s| T::extract_from(&s.profile));
            
            if let Some(profile) = profile {
                request.extensions_mut().insert(profile.clone());
                Ok(true)
            } else {
                Ok(false)
            }
        })().unwrap_or(false)
    }
}
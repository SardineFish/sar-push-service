use std::{ops::{Deref, DerefMut}, any::type_name};
use futures_util::future::{ok, err};

use actix_web::{FromRequest, HttpRequest, dev::Payload};
use futures::future::Ready;

pub struct ExtensionMove<T: ?Sized>(T);

impl<T: ?Sized> Deref for ExtensionMove<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized> DerefMut for ExtensionMove<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Sized + 'static> FromRequest for ExtensionMove<T> {
    type Config = ();
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, actix_web::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(t) = req.extensions_mut().remove::<T>() {
            ok(Self(t))
        } else {
            log::debug!(
                "Failed to extract extension. \
                Request path: {:?} (type: {})",
                req.path(),
                type_name::<T>(),
            );
            err(actix_web::error::ErrorInternalServerError(
                "Missing extension."
            ))
        }
    }
}
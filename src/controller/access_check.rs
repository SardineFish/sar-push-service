use crate::model::{Access, Error as ModelError, Model, UserProfile};
use actix_web::{error as web_errors, Result};
use futures::Future;
use std::pin::Pin;

const ERR_ACCESS_DENIED: &str = "Access denied";

fn handle_model_err(err: ModelError) -> actix_web::Error {
    match err {
        ModelError::NoRecord => web_errors::ErrorNotFound("User not found"),
        _ => web_errors::ErrorInternalServerError(err),
    }
}

type AccessCheckAsyncResult<'a> = Pin<Box<dyn Future<Output = Result<UserProfile>> + 'a>>;

pub trait AccessCheckUtils {
    fn allow_self_or_admin_access<'a>(
        &'a self,
        auth: &'a UserProfile,
        service_access: Access,
        uid: &'a str,
    ) -> AccessCheckAsyncResult<'a>;

    fn allow_admin_access<'a>(
        &'a self,
        service_access: Access,
        uid: &'a str,
    ) -> AccessCheckAsyncResult<'a>;
}

impl AccessCheckUtils for Model {
    fn allow_self_or_admin_access<'a>(
        &'a self,
        auth: &'a UserProfile,
        service_access: Access,
        uid: &'a str,
    ) -> AccessCheckAsyncResult<'a> {
        Box::pin(async move {
            let is_self = auth.uid == uid;
            if !is_self && service_access < Access::Admin {
                return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
            }

            let profile = self.get_profile(&uid).await.map_err(handle_model_err)?;

            if !is_self && service_access <= profile.access {
                Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED))
            } else {
                Ok(profile)
            }
        })
    }

    fn allow_admin_access<'a>(&'a self, service_access: Access, uid: &'a str) -> AccessCheckAsyncResult<'a> {
        Box::pin(async move {
            if service_access < Access::Admin {
                return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
            }

            let profile = self.get_profile(&uid).await.map_err(handle_model_err)?;

            if service_access <= profile.access {
                Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED))
            } else {
                Ok(profile)
            }
        })
    }

}

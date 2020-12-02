extern crate hex;
extern crate openssl;

use std::mem::{replace, swap};

use crate::model::{Access, Error as ModelError};
use crate::{
    middleware,
    model::{self, AccessManagerProfile},
};
use actix_web::{dev::Extensions, HttpRequest, HttpResponse, Responder, Result, delete, error as web_errors, get, http::StatusCode, patch, post, web};
use model::UserProfile;
use serde::{Deserialize, Serialize};
use web::{Data, Json, Path};
use super::extractor::ExtensionMove;

#[derive(Serialize)]
struct UserAccessProfile {
    uid: String,
    secret: String,
}

#[derive(Serialize, Deserialize)]
struct PublicUserProfile {
    name: String,
    description: String,
    access: Access,
}

impl From<model::UserProfile> for PublicUserProfile {
    fn from(profile: UserProfile) -> Self {
        Self {
            name: profile.name,
            description: profile.description,
            access: profile.access,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct UserProfileWithUID {
    uid: String,
    name: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct UserProfilePartial {
    name: Option<String>,
    description: Option<String>,
}

type Auth = ExtensionMove<model::UserProfile>;
type ServiceProfile = ExtensionMove<AccessManagerProfile>;
type Model = Data<model::Model>;

const ERR_ACCESS_DENIED: &str = "Access denied";

fn handle_model_err(err: ModelError) -> actix_web::Error {
    match err {
        ModelError::NoRecord => web_errors::ErrorNotFound("User not found"),
        _ => web_errors::ErrorInternalServerError(err),
    }
}

#[post("/user")]
async fn add_user(
    service: ServiceProfile,
    mut user: Json<PublicUserProfile>,
    model: Model,
) -> Result<Json<UserAccessProfile>> {
    let name = replace(&mut user.name, String::new());
    
    if service.access < user.access {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }

    let description = replace(&mut user.description, String::new());
    let profile = model.new_user(name, description, user.access);
    drop(user);

    let uid = profile.uid.clone();
    let secret = profile.secret.clone();

    model
        .add_profile(profile)
        .await
        .map_err(|err| web_errors::ErrorInternalServerError(err))?;

    Ok(Json(UserAccessProfile {
        uid: uid,
        secret: secret,
    }))
}

#[get("/user/{uid}")]
async fn get_profile(
    Path(uid): Path<String>,
    user: Auth,
    service: ServiceProfile,
    model: Model,
) -> Result<Json<PublicUserProfile>> {
    let profile = model.get_profile(&uid).await.map_err(handle_model_err)?;

    let is_self = user.uid == uid;
    let access_permit = service.access >= Access::Admin && service.access < profile.access;
    if !is_self && !access_permit {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }
    Ok(Json(PublicUserProfile::from(profile)))
}

#[patch("/user/{uid}")]
async fn update_profile(
    Path(uid): Path<String>,
    mut user: Json<UserProfilePartial>,
    auth: Auth,
    service: ServiceProfile,
    model: Model,
) -> Result<Json<PublicUserProfile>> {
    if auth.uid != uid && service.access < Access::Admin {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }

    let mut profile = model.get_profile(&uid).await.map_err(handle_model_err)?;

    if auth.uid != profile.uid && service.access < profile.access {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }

    if let Some(name) = &mut user.name {
        swap(&mut profile.name, name);
    }
    if let Some(desc) = &mut user.description {
        swap(&mut profile.description, desc);
    }

    let profile = model
        .update_profile(profile)
        .await
        .map_err(handle_model_err)?;

    Ok(Json(PublicUserProfile::from(profile)))
}

#[post("/user/{uid}/secret")]
async fn revoke_secret(
    Path(uid): Path<String>,
    auth: Auth,
    service: ServiceProfile,
    model: Model,
) -> Result<Json<UserAccessProfile>> {
    if auth.uid != uid && service.access < Access::Admin {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }

    let profile = model.get_profile(&uid).await.map_err(handle_model_err)?;
    if service.access < profile.access {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }

    let new_secret = model.revoke_secret(&uid).await.map_err(handle_model_err)?;

    Ok(Json(UserAccessProfile {
        uid: uid,
        secret: new_secret,
    }))
}

#[delete("/user/{uid}")]
async fn delete_user(
    Path(uid): Path<String>,
    request: HttpRequest,
    model: Model,
    service: ServiceProfile,
) -> Result<HttpResponse> {
    if service.access < Access::Admin {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }
    let profile;
    match model.get_profile(&uid).await {
        Ok(p) => {
            profile = p;
        }
        Err(ModelError::NoRecord) => {
            return Ok(HttpResponse::NoContent().finish());
        }
        Err(err) => {
            return Err(web_errors::ErrorInternalServerError(err));
        }
    }
    if service.access < profile.access {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }
    model.remove_user(&uid).await.map_err(handle_model_err)?;
    let response = Json(PublicUserProfile::from(profile)).with_status(StatusCode::OK)
        .respond_to(&request)
        .await?;
    Ok(response)
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(add_user);
}

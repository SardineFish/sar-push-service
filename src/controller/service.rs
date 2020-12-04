use actix_web::{
    delete, error as web_errors, get,
    http::StatusCode,
    patch, post,
    web::{self, Path},
    HttpRequest, HttpResponse, Responder, Result,
};
use model::{Access, Service, ServiceManagerProfile, UserProfile, ValidateProfile};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use web::Json;
use crate::utils::assert::*;

use crate::{model, model::Error as ModelError, utils::variant_eq};

use super::access_check::AccessCheckUtils;
use super::extractor::ExtensionMove;

#[derive(Serialize, Deserialize)]
struct ServiceProfileData {
    service_id: String,
    service: model::Service,
}

impl From<model::ServiceRecord> for ServiceProfileData {
    fn from(service: model::ServiceRecord) -> Self {
        Self {
            service_id: hex::encode(service._id.bytes()),
            service: service.service,
        }
    }
}

type Model = web::Data<model::Model>;
type ServiceProfile = ExtensionMove<ServiceManagerProfile>;
type Auth = ExtensionMove<UserProfile>;

const ERR_ACCESS_DENIED: &str = "Access denied";

fn handle_model_err(err: ModelError) -> actix_web::Error {
    match err {
        ModelError::NoRecord => web_errors::ErrorNotFound("User not found"),
        _ => web_errors::ErrorInternalServerError(err),
    }
}

#[get("/profile/{uid}")]
async fn get_profile(
    Path(uid): Path<String>,
    model: Model,
    auth: Auth,
    service: ServiceProfile,
) -> Result<Json<Vec<ServiceProfileData>>> {
    let profile: UserProfile = model.allow_self_or_admin_access(&auth, service.access, &uid).await?;

    let data = profile
        .services
        .into_iter()
        .map(|s| ServiceProfileData::from(s))
        .collect();

    Ok(Json(data))
}

#[post("/profile/{uid}")]
async fn add_service(
    Path(uid): Path<String>,
    model: Model,
    Json(data): Json<Service>,
    service: ServiceProfile,
) -> Result<Json<ServiceProfileData>> {
    let profile: UserProfile = model.allow_admin_access(service.access, &uid).await?;

    if profile
        .services
        .into_iter()
        .any(|s| variant_eq(&s.service, &data))
    {
        return Err(web_errors::ErrorConflict("Service already existed"));
    }

    if !data.validate_properties(&service) {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }

    let record = model
        .add_service(&uid, data)
        .await
        .map_err(|err| match err {
            ModelError::NoRecord => web_errors::ErrorConflict("Service Already existed"),
            err => handle_model_err(err),
        })?;

    Ok(Json(ServiceProfileData::from(record)))
}

#[patch("/profile/{uid}/{service_id}")]
async fn update_service(
    Path((uid, service_id)): Path<(String, String)>,
    model: Model,
    auth: Auth,
    Json(data): Json<model::Service>,
    service: ServiceProfile,
) -> Result<Json<ServiceProfileData>> {
    let profile: UserProfile = model
        .allow_self_or_admin_access(&auth, service.access, &uid)
        .await?;

    if !data.validate_properties(&service) {
        return Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED));
    }

    let service_id = ObjectId::with_string(&service_id)
        .map_err(|_| web_errors::ErrorBadRequest("Invalid service_id"))?;

    let mut service_profile = profile
        .services
        .into_iter()
        .find(|s| s._id == service_id)
        .ok_or(web_errors::ErrorNotFound("Service not found"))?;

    if variant_eq(&service_profile.service, &data) {
        service_profile.service = data;

        model
            .update_service(&uid, service_profile.clone())
            .await
            .map_err(handle_model_err)?;

        Ok(Json(ServiceProfileData::from(service_profile)))
    } else {
        Err(web_errors::ErrorBadRequest("Change of service type is forbidden"))
    }
}

#[delete("/profile/{uid}/{service_id}")]
async fn remove_service(
    Path((uid, service_id)): Path<(String, String)>,
    auth: Auth,
    model: Model,
    service: ServiceProfile,
    request: HttpRequest,
) -> Result<HttpResponse> {
    let profile: UserProfile = model
        .allow_self_or_admin_access(&auth, service.access, &uid)
        .await?;
    let service_id = match ObjectId::with_string(&service_id) {
        Ok(id) => id,
        Err(_) => { return Ok(HttpResponse::NoContent().finish()); }
    };
    let record = profile.services.into_iter().find(|s| s._id == service_id);
    if let Some(record) = record {
        match model.remove_service(&uid, record).await {
            Err(ModelError::NoRecord) => Ok(HttpResponse::NoContent().finish()),
            Err(err) => Err(handle_model_err(err)),
            Ok(record) => Ok(Json(ServiceProfileData::from(record))
                .with_status(StatusCode::OK)
                .respond_to(&request)
                .await?),
        }
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_profile)
        .service(add_service)
        .service(update_service)
        .service(remove_service);
}

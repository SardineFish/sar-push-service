use actix_web::{Result, error as web_errors, get, post, web::Data, web::Json, web::Path, web::{Query, ServiceConfig}};
use model::MailData;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use super::access_check::AccessCheckUtils;

use crate::model::{self, EmailNotify, ExtractProfile, NotifyProfile, NotifyState, UserProfile, Service};

use super::extractor::ExtensionMove;

#[derive(Deserialize)]
struct NotifyRequest {
    to: String,
    subject: String,
    content_type: String,
    body: String,
}

impl Into<MailData> for NotifyRequest {
    fn into(self) -> MailData {
        MailData {
            to: self.to,
            subject: self.subject,
            content_type: self.content_type,
            body: self.body,
        }
    }
}

#[derive(Deserialize)]
enum NotifyStatusFilter {
    All,
    Pending,
    Sent,
    Error,
}

#[derive(Deserialize)]
struct ListNotifyQuery {
    filter: NotifyStatusFilter,
}

#[derive(Serialize)]
enum NotifyStatus {
    Pending,
    Sent,
    Error,
}

#[derive(Serialize)]
struct PubNotifyInfo {
    message_id: String,
    status: NotifyStatus,
    error: Option<String>,
}

impl From<EmailNotify> for PubNotifyInfo {
    fn from(inner_notify: EmailNotify) -> Self {
        let mut notify = PubNotifyInfo {
            message_id: hex::encode(inner_notify._id.bytes()),
            status: NotifyStatus::Pending,
            error: None,
        };
        match inner_notify.status {
            NotifyState::Pending => {
                notify.status = NotifyStatus::Pending;
            }
            NotifyState::Sent => {
                notify.status = NotifyStatus::Sent;
            }
            NotifyState::Error(pub_err, _) => {
                notify.status = NotifyStatus::Error;
                notify.error = Some(pub_err);
            }
        }
        notify
    }
}

fn handel_model_error(err: model::Error) -> actix_web::Error {
    match err {
        model::Error::NoRecord => web_errors::ErrorNotFound("Notification not found"),
        err => web_errors::ErrorInternalServerError(err)
    }
    
}

type Auth = ExtensionMove<UserProfile>;
type ServiceProfile = ExtensionMove<NotifyProfile>;
type Model = Data<model::Model>;
type EmailNotifyService = Data<crate::service::EmailNotifyService>;

const ERR_ACCESS_DENIED: &str = "Access denied";

#[post("/queue")]
async fn queue(
    service: ServiceProfile,
    auth: Auth,
    Json(request): Json<NotifyRequest>,
    model: Model,
    push_service: EmailNotifyService,
) -> Result<Json<PubNotifyInfo>> {

    let record = auth
        .services
        .iter()
        .find(|s| NotifyProfile::extract_from(&s.service).is_some());

    log::debug!("Received request.");

    if let Some(record) = record {
        let service_id = record._id.clone();

        let notify =
            model.new_email_notify(service_id, request.into(), service.email_address.as_str());
        model
            .add_notification(&notify)
            .await
            .map_err(handel_model_error)?;

        push_service.enqueue().map_err(|err| web_errors::ErrorInternalServerError(err))?;

        Ok(Json(PubNotifyInfo::from(notify)))
    } else {
        Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED))
    }
}

#[get("/{message_id}")]
async fn query_status(Path(message_id): Path<String>, auth: Auth, model: Model) -> Result<Json<PubNotifyInfo>> {
    let message_id = ObjectId::with_string(message_id.as_str())
        .map_err(|_| web_errors::ErrorNotFound("Notification not found"))?;
    let notify = model.get_notification_by_message_id(&message_id)
        .await
        .map_err(handel_model_error)?;
    
    if !auth.services.iter().any(|s| s._id == notify.sender_profile) {
        Err(web_errors::ErrorForbidden(ERR_ACCESS_DENIED))
    } else {
        Ok(Json(PubNotifyInfo::from(notify)))
    }

}

#[get("/all/{uid}")]
async fn list_notifications(
    Path(uid): Path<String>, 
    auth: Auth, 
    model: Model, 
    Query(params): Query<ListNotifyQuery>,
) -> Result<Json<Vec<PubNotifyInfo>>> {
    let profile: UserProfile = model.allow_self_or_admin_access(&auth, auth.access, &uid).await?;

    let service_profile = profile.services.iter().find(|s| match s.service {
        Service::EmailNotify(_) => true,
        _ => false,
    }).ok_or(web_errors::ErrorNotFound("Service not found"))?;

    let filter = match params.filter {
        NotifyStatusFilter::All => |_: &EmailNotify| true,
        NotifyStatusFilter::Error => |t: &EmailNotify| t.status.is_error(),
        NotifyStatusFilter::Pending => |t: &EmailNotify| t.status.is_pending(),
        NotifyStatusFilter::Sent => |t: &EmailNotify| t.status.is_sent(),
    };

    let result = model.get_all_notifications_by_service(&service_profile._id)
        .await
        .map_err(handel_model_error)?;
    
    let result: Vec<PubNotifyInfo> = result.into_iter()
        .filter(filter)
        .map(PubNotifyInfo::from)
        .collect();

    Ok(Json(result))
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(queue)
        .service(query_status)
        .service(list_notifications);
}

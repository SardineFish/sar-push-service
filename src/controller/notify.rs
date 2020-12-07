use actix_web::{web::Query, Result, error as web_errors, get, post, web::Data, web::Json, web::ServiceConfig};
use model::MailData;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::model::{self, EmailNotify, ExtractProfile, NotifyProfile, NotifyState, UserProfile};

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
async fn query_status(message_id: Query<String>, model: Model) -> Result<Json<PubNotifyInfo>> {
    let message_id = ObjectId::with_string(message_id.as_str())
        .map_err(|_| web_errors::ErrorNotFound("Notification not found"))?;
    let notify = model.get_notification_by_message_id(&message_id)
        .await
        .map_err(handel_model_error)?;

    Ok(Json(PubNotifyInfo::from(notify)))
}

pub fn config(cfg: &mut ServiceConfig) {
    cfg.service(queue)
        .service(query_status);
}

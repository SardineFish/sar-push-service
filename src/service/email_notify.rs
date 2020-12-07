use model::{NotifyProfile, NotifyState, Service};
use mongodb::bson::oid::ObjectId;
use smtp::{mail::MailData, AuthCommand, Error as SMTPError, MIMEBody, MailBuilder, SMTPClient};
use std::{
    collections::VecDeque,
    fmt,
    sync::mpsc::{channel, Receiver, SendError, Sender},
    thread::spawn,
};

use crate::model::{self, EmailNotify, Model};

#[derive(Debug)]
enum Error {
    ModelError(model::Error),
    MissingServiceProfile,
    ConnectFailed(SMTPError),
    AuthError(SMTPError),
    SendError(SMTPError),
}

impl From<model::Error> for Error {
    fn from(err: model::Error) -> Self {
        Error::ModelError(err)
    }
}

// impl From<SMTPError> for Error {
//     fn from(err: SMTPError) -> Self {
//         Error::SMTPError(err)
//     }
// }

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::AuthError(_) => fmt::write(f, format_args!("SMTP Authorization failed")),
            Error::ConnectFailed(_) => fmt::write(f, format_args!("Cannot connect to SMTP Server")),
            Error::MissingServiceProfile => fmt::write(f, format_args!("Missing service profile")),
            Error::ModelError(_) => fmt::write(f, format_args!("Internal db error")),
            Error::SendError(SMTPError::ErrorReply(err)) => fmt::write(
                f,
                format_args!(
                    "Unexpected SMTP reply: {}: {}",
                    err.code,
                    err.text_lines.join("\r\n")
                ),
            ),
            Error::SendError(_) => fmt::write(f, format_args!("Internal SMTP error")),
        }
    }
}

pub struct MailSender {
    pub smtp_address: String,
    pub tls: bool,
    pub authuid: String,
    pub passwd: String,
}

#[derive(Clone)]
pub struct EmailNotifyService {
    model: Model,
    mail_sender: Sender<()>,
}

impl EmailNotifyService {
    pub fn new(model: Model) -> Self {
        let (sender, receiver) = channel::<()>();
        let model_cloned = model.clone();

        spawn(move || Self::notify_loop(receiver, model_cloned));

        Self {
            model: model,
            mail_sender: sender,
        }
    }

    pub fn enqueue(&self, _notify: EmailNotify) -> Result<(), SendError<()>> {
        self.mail_sender.send(())
    }

    fn build_mail(notify: &EmailNotify, profile: &NotifyProfile) -> MailData {
        let content = MIMEBody::new(&notify.mail.content_type).text(&notify.mail.body);

        let mail = MailBuilder::new()
            .from((&profile.name, &profile.password))
            .to(notify.mail.to.as_str())
            .message_id(&notify.message_id)
            .subject(&notify.mail.subject)
            .body(content)
            .build();

        mail
    }

    fn send_notify(notify: &EmailNotify, profile: &NotifyProfile) -> Result<(), Error> {
        let mail = Self::build_mail(&notify, &profile);

        let _client = SMTPClient::connect(&profile.smtp_address)
            .map_err(|err| Error::ConnectFailed(err))?
            .auth(AuthCommand::Plain(
                None,
                profile.username.clone(),
                profile.password.clone(),
            ))
            .map_err(|err| Error::AuthError(err))?
            .send(&profile.email_address, &notify.mail.to, mail)
            .map_err(|err| Error::SendError(err))?
            .quit()
            .err();

        Ok(())
    }

    fn send_notify_tls(notify: &EmailNotify, service_profile: &NotifyProfile) -> Result<(), Error> {
        Ok(())
    }

    async fn dequeue_notify(model: &Model) -> Result<(), Error> {
        let notifications = model.get_notifications().await.map_err(Error::from)?;

        let notify = notifications
            .into_iter()
            .find(|n| n.status == NotifyState::Pending);

        if let Some(notify) = notify {
            let service_profile = model
                .get_service_by_id(&notify.sender_profile)
                .await
                .map_err(|err| match err {
                    model::Error::NoRecord => Error::MissingServiceProfile,
                    err => err.into(),
                })?;

            let result = match service_profile {
                Service::EmailNotify(service_profile) if service_profile.tls => {
                    Self::send_notify_tls(&notify, &service_profile)
                }
                Service::EmailNotify(service_profile) => {
                    Self::send_notify(&notify, &service_profile)
                }
                _ => Err(Error::MissingServiceProfile),
            };

            let mut notify = notify;
            notify.status = match result {
                Ok(_) => NotifyState::Sent,
                Err(err) => {
                    log::warn!("Failed to send an email notify {:?}", err);
                    NotifyState::Error(format!("{}", err), format!("{:?}", err))
                }
            };

            model
                .update_notification(&notify)
                .await
                .map_err(Error::from)?;

            Ok(())
        } else {
            Ok(())
        }
    }

    fn notify_loop(receiver: Receiver<()>, model: Model) {
        let mut rt = actix_web::rt::Runtime::new().unwrap();
        rt.block_on(async move {
            loop {
                if let Err(_) = receiver.recv() {
                    break;
                }

                if let Err(err) = Self::dequeue_notify(&model).await {
                    log::error!("{:?}", err);
                }
            }
        });
    }
}

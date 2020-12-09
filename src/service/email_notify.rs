use model::{NotifyProfile, NotifyState, Service};
use smtp::{AuthCommand, Error as SMTPError, MIMEBody, MailBuilder, SMTPClient, SMTPClientTCP, SMTPClientTLS, mail::MailData};
use std::{fmt, sync::mpsc::{channel, Receiver, SendError, Sender}, thread::spawn, time::Duration};

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

#[derive(Clone)]
pub struct EmailNotifyService {
    mail_sender: Sender<()>,
}

impl EmailNotifyService {
    pub fn new(model: Model, timeout: Duration) -> Self {
        let (sender, receiver) = channel::<()>();

        let service = PushService {
            timeout,
            model,
            notify_receiver: receiver,
        };
        spawn(move || service.start());

        Self {
            mail_sender: sender,
        }
    }

    pub fn enqueue(&self) -> Result<(), SendError<()>> {
        self.mail_sender.send(())
    }

    
}


struct PushService {
    timeout: Duration,
    model: Model,
    notify_receiver: Receiver<()>,
}

impl PushService {
    
    fn start(&self) {
        let mut rt = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async move {
            log::info!("Email notify serice up");
            log::debug!("Start processing existed notifications");
            if let Err(err) = self.dequeue_notify().await {
                log::error!("{:?}", err);
            }

            loop {
                log::debug!("Waiting for notify");
                if let Err(_) = self.notify_receiver.recv() {
                    log::info!("Shutting down notify service");
                    break;
                }

                log::debug!("Start sending notification");
                if let Err(err) = self.dequeue_notify().await {
                    log::error!("{:?}", err);
                }
            }
        });
    }
    
    async fn dequeue_notify(&self) -> Result<(), Error> {
        let notifications = self.model.get_pending_notifications().await.map_err(Error::from)?;
        
        let iter = notifications
            .into_iter()
            .filter(|n| n.status == NotifyState::Pending);

        for mut notify in iter {

            let result = self.try_send_notify(&notify).await;

            notify.status = match result {
                Ok(_) => {
                    log::debug!("Notification email sent");
                    NotifyState::Sent
                },
                Err(err) => {
                    log::warn!("Failed to send an email notify {:?}", err);
                    NotifyState::Error(format!("{}", err), format!("{:?}", err))
                }
            };

            self.model
                .update_notification(&notify)
                .await
                .map_err(Error::from)?;
            
            log::debug!("Notification updated");
        }

        Ok(())
    }

    async fn try_send_notify(&self, notify: &EmailNotify) -> Result<(), Error> {
        log::debug!("Try sending notification to {}", &notify.mail.to);
        let service_profile = self.model
            .get_service_by_id(&notify.sender_profile)
            .await
            .map_err(|err| match err {
                model::Error::NoRecord => Error::MissingServiceProfile,
                err => err.into(),
            })?;

        let result = match service_profile {
            Service::EmailNotify(service_profile) if service_profile.tls => {
                let client = self.connect_tls(&service_profile)?;
                self.send_notify(&notify, &service_profile, client)
            }
            Service::EmailNotify(service_profile) => {
                let client = self.connect_smtp(&service_profile)?;
                self.send_notify(&notify, &service_profile, client)
            }
            _ => Err(Error::MissingServiceProfile),
        };

        result
    }

    

    fn build_mail(notify: &EmailNotify, profile: &NotifyProfile) -> MailData {
        let content = MIMEBody::new(&notify.mail.content_type).text(&notify.mail.body);

        let mail = MailBuilder::new()
            .from((&profile.name, &profile.email_address))
            .to(notify.mail.to.as_str())
            .message_id(&notify.message_id)
            .subject(&notify.mail.subject)
            .body(content)
            .build();

        mail
    }

    fn send_notify<S: std::io::Read + std::io::Write>(&self, notify: &EmailNotify, profile: &NotifyProfile, mut client: SMTPClient<S>) -> Result<(), Error> {
        let mail = Self::build_mail(&notify, &profile);
        client.auth(AuthCommand::Plain(
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

    fn connect_smtp(&self, profile: &NotifyProfile) -> Result<SMTPClientTCP, Error> {
        let client = SMTPClient::connect_timeout(&profile.smtp_address, self.timeout)
            .map_err(|err| Error::ConnectFailed(err))?;
        Ok(client)
    }
    
    fn connect_tls(&self, profile: &NotifyProfile) -> Result<SMTPClientTLS, Error> {
        let client = SMTPClient::connect_tls_timeout(&profile.smtp_address, self.timeout)
            .map_err(|err| Error::ConnectFailed(err))?;
        Ok(client)
    }
}
use super::helper::*;
use clap::{App, ArgMatches};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{error::Error, error::Result, AppConfig};

#[derive(Serialize, Deserialize, Debug)]
struct NotifyRequest {
    to: String,
    subject: String,
    content_type: String,
    body: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum NotifyStatus {
    Pending,
    Sent,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
struct PubNotifyInfo {
    message_id: String,
    status: NotifyStatus,
    error: Option<String>,
}

pub fn config<'s>() -> App<'s> {
    App::new("notify")
        .about("Email notification push service")
        .arg("[MSG_ID] 'message_id of a notification'")
        .subcommand(
            App::new("list")
                .about("List notifications")
                .arg("--all")
                .arg("--error")
                .arg("--sent")
                .arg("--pending")
                .arg("--user=[UID], 'User's uid to be list'")
        )
        .subcommand(
            App::new("send")
                .about("Send a noficiation through email")
                .arg("[RECEIVER_ADDR] 'Email address of the notification receiver'")
                .arg("[BODY_FILE] 'File path to the notification body'")
                .arg("--to=[RECEIVER_ADDR] 'Email address of the notification receiver'")
                .arg("--subject=[SUBJECT] 'Subject of the notification mail'")
                .arg("--content-type=[CONTENT_TYPE] 'Content-Type of the notification mail'")
                .arg("--text=[TEXT_BODY] 'Notification body text'"),
        )
}

pub async fn notify(cfg: AppConfig<'_>, matches: &ArgMatches) -> Result<()> {
    if let Some(matches) = matches.subcommand_matches("list") {
        let filter = if matches.is_present("all") {
            "All"
        } else if matches.is_present("error") {
            "Error"
        } else if matches.is_present("sent") {
            "Sent"
        } else if matches.is_present("pending") {
            "Pending"
        } else {
            "All"
        };

        let uid = if let Some(uid) = matches.value_of("user") {
            uid.to_string()
        } else if let Some(auth) = &cfg.auth {
            auth.uid.clone()
        } else {
            return Err(Error::ErrorInfo("Missing user"));
        };

        let result: Vec<PubNotifyInfo> = Client::new()
            .get(&format!("{}/notify/all/{}?filter={}", cfg.url, uid, filter))
            .auth(cfg.auth)
            .send()
            .await
            .map_err(Error::from)?
            .handle_error()
            .await?
            .json()
            .await
            .map_err(Error::from)?;
        
        println!("List notifications:");
        output(result, cfg.output);

    } else if let Some(matches) = matches.subcommand_matches("send") {
        let receiver = matches
            .value_of("RECEIVER_ADDR")
            .ok_or(Error::ErrorInfo("Missing receiver's mail address"))?
            .to_string();
        let content_type = matches
            .value_of("content-type")
            .unwrap_or("text/plain")
            .to_string();
        let subject = matches.value_of("subject").ok_or(Error::ErrorInfo("Missing notification subject"))?.to_string();
        let body = if let Some(text) = matches.value_of("text") {
            text.to_string()
        } else if let Some(file) = matches.value_of("BODY_FILE") {
            std::fs::read_to_string(file).map_err(Error::from)?
        } else {
            return Err(Error::ErrorInfo("Missing notification mail body"));
        };

        let request = NotifyRequest {
            to: receiver,
            content_type,
            body,
            subject,
        };

        let result: PubNotifyInfo = Client::new()
            .post(&format!("{}/notify/queue", cfg.url))
            .auth(cfg.auth)
            .json(&request)
            .send()
            .await
            .map_err(Error::from)?
            .handle_error()
            .await?
            .json()
            .await
            .map_err(Error::from)?;

        println!("Notification queued.");
        output(result, cfg.output);

    } else if let Some(msg_id) = matches.value_of("MSG_ID") {
        let result: PubNotifyInfo = Client::new()
            .get(&format!("{}/notify/{}", cfg.url, msg_id))
            .auth(cfg.auth)
            .send()
            .await
            .map_err(Error::from)?
            .handle_error()
            .await?
            .json()
            .await
            .map_err(Error::from)?;

        println!("Notification found.");
        output(result, cfg.output);
    } else {
        return Err(Error::ErrorInfo("Invalid arguments"));
    }

    Ok(())
}

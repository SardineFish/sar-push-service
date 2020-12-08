use auth::UserAuth;
use clap::{App, Subcommand, Arg};
use tokio::prelude::*;

extern crate clap;
extern crate reqwest;
extern crate serde;
extern crate serde_yaml;

mod auth;
mod access;
mod service;
mod notify;
mod helper;

#[derive(Debug, Default)]
pub struct AppConfig<'s> {
    output: Option<&'s str>,
    url: &'s str,
    auth: Option<UserAuth>,
}

#[tokio::main]
async fn main() {
    let matches = App::new("Sar Push Service Client")
        .version("0.1.0")
        .author("SardineFish")
        .about("Client connect to Sar Push Service")
        .arg("--auth=[FILE] 'Import authorization info from file'")
        .arg("--uid=[UID] 'uid used for authorization'")
        .arg("--secret=[SECRET] 'secret used for authorization'")
        .arg("-o, --output=[OUTPUT] 'Save output to file'")
        .arg("[URL] 'API url'")
        .subcommand(access::config())
        .subcommand(App::new("service")
            .about("Service management")
            .arg("--user=[UID] 'uid of a specific user'")
            .arg("--service=[SID] 'service_id of a specific service'")
            .arg("--add 'Add new service profile'")
            .arg("-d, --delete 'Delete a service from a user specific by uid'")
            .subcommand(App::new("notify")
                .about("Email notification push service")
                .arg("--smtp-addr=[SMTP_ADDR] 'Address of the SMTP server'")
                .arg("--tls 'Wether connect to SMTP server through TLS'")
                .arg("--username=[USRNAME] 'Username used for SMTP authorization'")
                .arg("--password=[PASSWD] 'Password used for SMTP authorization'")
                .arg("--email-addr=[MAIL_ADDR] 'Mail address of the notification sender'")
                .arg("--name=[NAME] 'Display name of the noficiation sender'")
            )
            .subcommand(App::new("access")
                .about("User access management service.")
                .arg("--access=[ACCESS] 'Access level of the access manager'")
            )
            .subcommand(App::new("service")
                .about("Service management")
                .arg("--access=[ACCESS] 'Access level of the service mamager'")
            )
        )
        .subcommand(App::new("notify")
            .about("Email notification push service")
            .arg("[MSG_ID] 'message_id of a notification'")
            .arg("--list 'List notifications'")
            .arg("--all ")
            .arg("--error")
            .arg("--sent")
            .arg("--pending")
            .subcommand(App::new("send")
                .about("Send a noficiation through email")
                .arg("[EMAIL_ADDR] 'Email address of the notification receiver'")
                .arg("[BODY_FILE] 'File path to the notification body'")
                .arg("--content-type=[CONTENT_TYPE] 'Content-Type of the notification mail'")
                .arg("--text=[TEXT_BODY] 'Notification body text'")
            )
        )
        .get_matches();

    let mut config = AppConfig::default();

    if let Some(auth_file) = matches.value_of("auth") {
        let data = std::fs::read_to_string(auth_file).unwrap();
        config.auth = Some(serde_json::from_str(&data).unwrap());
    } else if let (Some(uid), Some(secret)) = (matches.value_of("uid"), matches.value_of("secret")) {
        config.auth = Some(UserAuth {
            uid: uid.to_string(),
            secret: secret.to_string()
        });
    }
    
    config.output = matches.value_of("output");

    if let Some(url) = matches.value_of("URL") {
        config.url = url;
    } else {
        config.url = "http://localhost/";
    }


    if let Some(matches) = matches.subcommand_matches("access") {
        access::access(config, matches).await;
    }

}

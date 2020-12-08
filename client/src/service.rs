use clap::{App, ArgMatches};
use reqwest::Client;
use serde::{Serialize, Deserialize};
use super::access::Access;
use super::helper::*;

use crate::{AppConfig, error::Error, error::Result};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content="profile")]
enum Service {
    UserAccessControl(AccessManagerProfile),
    EmailNotify(NotifyProfile),
    ServiceManagement(ServiceManagerProfile),
}

#[derive(Debug, Serialize, Deserialize)]
struct AccessManagerProfile {
    pub access: Access,
}

#[derive(Debug, Serialize, Deserialize)]
struct NotifyProfile {
    pub smtp_address: String,
    pub tls: bool,
    pub username: String,
    pub password: String,
    pub email_address: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServiceManagerProfile {
    pub access: Access,
}

#[derive(Serialize, Deserialize, Debug)]
struct ServiceProfile {
    pub service_id: String,
    pub service: Service,
}

pub fn config<'s>() -> App<'s> {
    App::new("service")
        .about("Service management")
        .arg("--user=[UID] 'uid of a specific user'")
        .arg("--service=[SID] 'service_id of a specific service'")
        .arg("--add 'Add new service profile'")
        .arg("--update 'Update a service profile'")
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
}

pub async fn service(cfg: AppConfig<'_>, matches: &ArgMatches) -> Result<()> {
    if matches.is_present("add") {

        let uid = matches.value_of("user").ok_or(Error::ErrorInfo("Missing 'user'"))?;
        let profile = build_service_profile(&matches)?;

        let result: ServiceProfile = Client::new()
            .post(&format!("{}/service/profile/{}", cfg.url, uid))
            .auth(cfg.auth)
            .json(&profile)
            .send()
            .await
            .map_err(Error::from)?
            .handle_error()
            .await?
            .json()
            .await
            .map_err(Error::from)?;

        println!("Service profile added.");
        output(result, cfg.output);
    } else if matches.is_present("delete") {
        let uid = matches.value_of("user").ok_or(Error::ErrorInfo("Missing 'user'"))?;
        let service_id = matches.value_of("service").ok_or(Error::ErrorInfo("Missing 'service'"))?;

        let result: ServiceProfile = Client::new()
            .delete(&format!("{}/service/profile/{}/{}", cfg.url, uid, service_id))
            .auth(cfg.auth)
            .send()
            .await
            .map_err(Error::from)?
            .handle_error()
            .await?
            .json()
            .await
            .map_err(Error::from)?;
        
        println!("Service profile deleted.");
        output(result, cfg.output);
    } else if matches.is_present("update") {
        let uid = matches.value_of("user").ok_or(Error::ErrorInfo("Missing 'user'"))?;
        let service_id = matches.value_of("service").ok_or(Error::ErrorInfo("Missing 'service'"))?;
        let profile = build_service_profile(&matches)?;

        let result: ServiceProfile = Client::new()
            .patch(&format!("{}/service/profile/{}/{}", cfg.url, uid, service_id))
            .auth(cfg.auth)
            .json(&profile)
            .send()
            .await
            .map_err(Error::from)?
            .handle_error()
            .await?
            .json()
            .await
            .map_err(Error::from)?;

        println!("Service profile updated.");
        output(result, cfg.output);
    } else if let Some(uid) = matches.value_of("user") {
        if let Some(service_id) = matches.value_of("service") {
            let result: ServiceProfile = Client::new()
                .get(&format!("{}/service/profile/{}/{}", cfg.url, uid, service_id))
                .auth(cfg.auth)
                .send()
                .await
                .map_err(Error::from)?
                .handle_error()
                .await?
                .json()
                .await
                .map_err(Error::from)?;

            output(result, cfg.output);
        } else {
            let result: Vec<ServiceProfile> = Client::new()
                .get(&format!("{}/service/profile/{}", cfg.url, uid))
                .auth(cfg.auth)
                .send()
                .await
                .map_err(Error::from)?
                .handle_error()
                .await?
                .json()
                .await
                .map_err(Error::from)?;

            println!("All Services:");
            output(result, cfg.output);
        }
    }

    Ok(())
}

fn build_service_profile(matches: &ArgMatches) -> Result<Service> {
    if let Some(matches) = matches.subcommand_matches("notify") {
        let profile = NotifyProfile {
            smtp_address: matches.value_of("smtp-addr").ok_or(Error::ErrorInfo("Missing 'smtp-addr'"))?.to_string(),
            tls: matches.is_present("tls"),
            username: matches.value_of("username").ok_or(Error::ErrorInfo("Missing 'username'"))?.to_string(),
            password: matches.value_of("password").ok_or(Error::ErrorInfo("Missing 'password'"))?.to_string(),
            email_address: matches.value_of("email-addr").ok_or(Error::ErrorInfo("Missing 'email-addr'"))?.to_string(),
            name: matches.value_of("name").ok_or(Error::ErrorInfo("Missing 'name'"))?.to_string(),
        };
        Ok(Service::EmailNotify(profile))
    } else if let Some(matches) = matches.subcommand_matches("access") {
        let profile = AccessManagerProfile {
            access: Access::from_str(matches.value_of("access").ok_or(Error::ErrorInfo("Missing 'access'"))?)?
        };
        Ok(Service::UserAccessControl(profile))
    } else if let Some(matches) = matches.subcommand_matches("service") {
        Ok(Service::ServiceManagement(ServiceManagerProfile {
            access: Access::from_str(matches.value_of("access").ok_or(Error::ErrorInfo("Missing 'access'"))?)?
        }))
    } else {
        Err(Error::ErrorInfo("Unknown service type"))
    }
}
use super::helper::*;
use clap::{App, ArgMatches};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{AppConfig, error::Error, auth::UserAuth, error::Result};

#[derive(Serialize, Deserialize, Debug)]
pub enum Access {
    Root,
    Admin,
    User,
}

impl Access {
    pub fn from_str(access: &str) -> Result<Self> {
        serde_yaml::from_str(access)
            .map_err(|_| Error::ErrorInfo("Invalid access"))
    }
}

#[derive(Serialize, Deserialize)]
struct PubUserInfo {
    pub name: String,
    pub description: String,
    pub access: Access,
}

pub fn config() -> App<'static> {
    App::new("access")
        .about("User access controller.")
        .arg("--name=[NAME] 'Name of the user'")
        .arg("--description=[DESC] 'Description of the user'")
        .arg("--access=[ACCESS] 'Access level of the user'")
        .arg("-d, --delete 'Delete a user'")
        .arg("[UID] 'uid of a specific user'")
}

pub async fn access<'s>(cfg: AppConfig<'s>, matches: &ArgMatches) -> Result<()> {
    if let Some(uid) = matches.value_of("UID") {
        if matches.is_present("delete") {
            let response: UserAuth = Client::new()
                .delete(format!("{}/access/user/{}", cfg.url, uid).as_str())
                .auth(cfg.auth)
                .send()
                .await
                .map_err(Error::from)?
                .handle_error()
                .await?
                .json()
                .await
                .map_err(Error::from)?;

            output(response, cfg.output);
        } else {
            let response: PubUserInfo = Client::new()
                .get(format!("{}/access/user/{}", cfg.url, uid).as_str())
                .auth(cfg.auth)
                .send()
                .await
                .map_err(Error::from)?
                .handle_error()
                .await?
                .json()
                .await
                .map_err(Error::from)?;

            output(response, cfg.output);
        }
    } else if let (Some(name), Some(desc), Some(access)) = (
        matches.value_of("name"),
        matches.value_of("description"),
        matches.value_of("access"),
    ) {
        let access: Access = serde_yaml::from_str(access).map_err(|_| Error::ErrorInfo("Invalid access"))?;
        let profile = PubUserInfo {
            name: name.to_string(),
            description: desc.to_string(),
            access: access
        };

        let result: UserAuth = Client::new()
            .post(&format!("{}/access/user", cfg.url))
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
        output(result, cfg.output);
    }

    Ok(())
}

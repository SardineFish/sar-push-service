use super::helper::*;
use clap::{App, ArgMatches};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{auth::UserAuth, AppConfig};

#[derive(Serialize, Deserialize)]
pub enum Access {
    Root,
    Admin,
    User,
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

pub async fn access<'s>(cfg: AppConfig<'s>, matches: &ArgMatches) {
    if let Some(uid) = matches.value_of("UID") {
        if matches.is_present("delete") {
            let response: UserAuth = Client::new()
                .delete(format!("{}/access/user/{}", cfg.url, uid).as_str())
                .auth(cfg.auth)
                .send()
                .await
                .unwrap()
                .handle_error()
                .await
                .json()
                .await
                .unwrap();

            output(response, cfg.output);
        } else {
            let response: PubUserInfo = Client::new()
                .get(format!("{}/access/user/{}", cfg.url, uid).as_str())
                .auth(cfg.auth)
                .send()
                .await
                .unwrap()
                .handle_error()
                .await
                .json()
                .await
                .unwrap();

            output(response, cfg.output);
        }
    } else if let (Some(name), Some(desc), Some(access)) = (
        matches.value_of("name"),
        matches.value_of("description"),
        matches.value_of("access"),
    ) {
        let access: Access = serde_yaml::from_str(access).expect("Invalid access");
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
            .unwrap()
            .handle_error()
            .await
            .json()
            .await
            .unwrap();
        output(result, cfg.output);
    }
}

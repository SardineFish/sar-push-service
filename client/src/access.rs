use super::helper::*;
use clap::{App, ArgMatches};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{auth::UserAuth, error::Error, error::Result, AppConfig};

#[derive(Serialize, Deserialize, Debug)]
pub enum Access {
    Root,
    Admin,
    User,
}

impl Access {
    pub fn from_str(access: &str) -> Result<Self> {
        serde_yaml::from_str(access).map_err(|_| Error::ErrorInfo("Invalid access"))
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
        .arg("[UID] 'uid of the user to query'")
        .subcommand(
            App::new("revoke")
                .about("Revoke and regenerate user secret")
                .arg("[UID] 'uid of a user to revoke secret'"),
        )
        .subcommand(
            App::new("delete")
                .about("Delete a user")
                .arg("[UID] 'uid of a user to delete'"),
        )
        .subcommand(
            App::new("grant")
                .alias("new")
                .alias("add")
                .alias("create")
                .about("Grant a new access user")
                .arg("--name=[NAME] 'Name of the user'")
                .arg("--description=[DESC] 'Description of the user'")
                .arg("--access=[ACCESS] 'Access of the user'"),
        )
        .subcommand(
            App::new("update")
                .alias("edit")
                .about("Update a user profile")
                .arg("[UID] 'uid of a user to update'")
                .arg("--name=[NAME] 'Name of the user'")
                .arg("--description=[DESC] 'Description of the user'")
                .arg("--access=[ACCESS] 'Access of the user'"),
        )
}

pub async fn access<'s>(cfg: AppConfig<'s>, matches: &ArgMatches) -> Result<()> {
    if let Some(matches) = matches.subcommand_matches("revoke") {
        revoke(cfg, matches).await?;
    } else if let Some(matches) = matches.subcommand_matches("delete") {
        delete(cfg, matches).await?;
    } else if let Some(matches) = matches.subcommand_matches("grant") {
        grant(cfg, matches).await?;
    } else if let Some(uid) = matches.value_of("UID") {
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
    } else {
        return Err(Error::ErrorInfo("Invalid arguments"));
    }

    Ok(())
}

async fn delete(cfg: AppConfig<'_>, matches: &ArgMatches) -> Result<()> {
    let uid = matches
        .value_of("UID")
        .ok_or(Error::ErrorInfo("Missing UID"))?;
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

    println!("User deleted.");
    output(response, cfg.output);
    Ok(())
}

async fn revoke(cfg: AppConfig<'_>, matches: &ArgMatches) -> Result<()> {
    let uid = if let Some(uid) = matches.value_of("UID") {
        uid
    } else if let Some(auth) = &cfg.auth {
        &auth.uid
    } else {
        return Err(Error::ErrorInfo("Missing uid"));
    };

    let response: UserAuth = Client::new()
        .post(&format!("{}/access/user/{}/secret", cfg.url, uid))
        .auth(cfg.auth)
        .send()
        .await
        .map_err(Error::from)?
        .handle_error()
        .await?
        .json()
        .await
        .map_err(Error::from)?;

    println!("A new secret successfully generated.");
    output(response, cfg.output);
    Ok(())
}

async fn grant(cfg: AppConfig<'_>, matches: &ArgMatches) -> Result<()> {
    let name = matches
        .value_of("name")
        .ok_or(Error::ErrorInfo("Missing user name"))?;
    let desc = matches
        .value_of("description")
        .ok_or(Error::ErrorInfo("Missing user description"))?;
    let access = matches
        .value_of("access")
        .ok_or(Error::ErrorInfo("Missing user access"))?;
    let access = Access::from_str(access)?;

    let profile = PubUserInfo {
        name: name.to_string(),
        description: desc.to_string(),
        access: access,
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

    println!("A new access granted.");
    output(result, cfg.output);
    Ok(())
}

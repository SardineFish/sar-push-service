use auth::UserAuth;
use clap::{App, };

extern crate clap;
extern crate reqwest;
extern crate serde;
extern crate serde_yaml;

mod auth;
mod access;
mod service;
mod notify;
mod helper;
mod error;

use error::Error;

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
        .subcommand(service::config())
        .subcommand(notify::config())
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
        config.url = "http://localhost:5000";
    }


    let result = if let Some(matches) = matches.subcommand_matches("access") {
        access::access(config, matches).await
    } else if let Some(matches) = matches.subcommand_matches("service") {
        service::service(config, matches).await
    } else if let Some(matches) = matches.subcommand_matches("notify") {
        notify::notify(config, &matches).await
    }
    else {
        Ok(())
    };

    match result {
        Err(Error::JsonError(err)) => println!("Invalid JSON format: {:?}", err),
        Err(Error::NetworkError(err)) => println!("API Request failed: {:?}", err),
        Err(Error::ResponseError(status, err)) => println!("Error {} {}", status, err),
        Err(Error::ErrorInfo(err)) => println!("{}", err),
        _ => ()
    }

}

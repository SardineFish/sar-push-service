use std::io::{Write, Read};

use crate::{command::SMTPCommand, extension::Extension, error::{Result, Error}, smtp::SMTPClient};
use base64;

#[derive(Default)]
pub struct Auth {
    supported_mechanism: Vec<Mechanism>,
}

impl Auth {
    pub fn send_auth<S: Write + Read>(client: &mut SMTPClient<S>, mechanisum: AuthCommand) -> Result<()> {
        let reply = client.send_command(mechanisum)?;
        if reply.code == 235 {
            Ok(())
        } else {
            Err(Error::ErrorReply(reply))
        }
    }
}

impl Extension for Auth {
    fn name() -> &'static str {
        "AUTH"
    }
    fn register(params: &[&str]) -> Self {
        let mut auth = Self::default();
        for param in params {
            match Mechanism::from(*param) {
                Mechanism::Unknown => continue,
                mechanisum => auth.supported_mechanism.push(mechanisum),
            }
        }
        auth
    }
}

pub enum Mechanism {
    Plain,
    Login,
    Unknown,
}


impl From<&str> for Mechanism {
    fn from(name: &str) -> Self {
        match name.to_uppercase().as_str() {
            "PLAIN" => Mechanism::Plain,
            "LOGIN" => Mechanism::Login,
            _ => Mechanism::Unknown,
        }
    }
}


pub enum AuthCommand {
    Plain(Option<String>, String, String),
}


impl SMTPCommand for AuthCommand {
    fn command(&self) -> &'static str {
        "AUTH"
    }
    fn params(&self) -> Option<String> {
        match self {
            AuthCommand::Plain(authzid, authcid, passwd) => {
                let authzid: &str = authzid.as_ref().map(|s|s.as_str()).unwrap_or("");
                let params = format!("{}\0{}\0{}", authzid, authcid, passwd);
                Some(base64::encode(params))
            }
        }
    }
}
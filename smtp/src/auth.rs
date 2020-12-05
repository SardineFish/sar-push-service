use std::io::{Write, Read};

use crate::{command::SMTPCommand, error::{Result, Error}, extension::Extension, smtp::{SMTPInner, Stream}};
use base64;

pub struct Auth<'s, S: Stream> {
    supported_mechanism: Vec<Mechanism>,
    smtp: &'s mut SMTPInner<S>,
}

impl<'s, S: Stream> Auth<'s, S> {
    pub fn send_auth(&mut self, mechanisum: AuthCommand) -> Result<()> {
        let reply = self.smtp.send_command(mechanisum)?;
        if reply.code == 235 {
            Ok(())
        } else {
            Err(Error::ErrorReply(reply))
        }
    }
}

impl<'s, S: Stream> Extension<'s, S> for Auth<'s, S> {
    fn name() -> &'static str {
        "AUTH"
    }
    fn register(smtp: &'s mut SMTPInner<S>, params: &[String]) -> Self {
        
        let mut auth = Self {
            supported_mechanism: Default::default(),
            smtp: smtp,
        };
        for param in params {
            match Mechanism::from(param.as_str()) {
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
                Some(format!("PLAIN {}", base64::encode(params)))
            }
        }
    }
}
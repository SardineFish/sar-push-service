use crate::{extension::Extension, command::SMTPCommand};

#[derive(Default)]
pub struct Auth {
    supported_mechanism: Vec<Mechanism>,
}

impl Auth {

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


pub enum AuthCommand {
    Plain(Option<String>, String, String),
}


impl SMTPCommand for AuthCommand {
    fn command(&self) -> &'static str {
        "AUTH"
    }
    fn params(&self) -> Option<String> {
        match self {

        }
    }
}
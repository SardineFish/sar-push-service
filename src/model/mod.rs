extern crate mongodb;
extern crate tokio;

mod profile;
mod error;
mod notify;
mod init;
mod service;

use std::time::Duration;

use mongodb::{ Client, options::ClientOptions };

#[derive(Clone)]
pub struct Model {
    mongo: mongodb::Client,
    db: mongodb::Database,
}

const DB_TIMEOUT: u64 = 1;

impl Model {
    pub async fn new() -> Result<Self, mongodb::error::Error> {
        let mut options = ClientOptions::parse("mongodb://mongo").await?;
        options.connect_timeout = Some(Duration::from_secs(DB_TIMEOUT));
        options.server_selection_timeout = Some(Duration::from_secs(DB_TIMEOUT));
        let client = Client::with_options(options)?;
        let db = client.database("sar-notify");
        Ok(Model {
            mongo: client,
            db: db,
        })
    }
}

pub use profile::{ Profile, Access, Service, ServiceRecord, ExtractProfile };
pub use error::{ Error };
pub use notify::NotifyProfile;
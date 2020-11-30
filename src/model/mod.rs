extern crate mongodb;
extern crate tokio;

mod profile;
mod error;
mod notify;
mod init;
mod service;

use mongodb::{ Client, options::ClientOptions };

pub struct Model {
    mongo: mongodb::Client,
    db: mongodb::Database,
}

impl Model {
    pub async fn new() -> Result<Self, mongodb::error::Error> {
        let options = ClientOptions::parse("mongodb://mongo").await?;
        let client = Client::with_options(options)?;
        let db = client.database("sar-notify");
        Ok(Model {
            mongo: client,
            db: db,
        })
    }
}
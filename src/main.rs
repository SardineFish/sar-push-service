#![feature(trait_alias)]

extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate futures_util;
extern crate log;

mod controller;
mod middleware;
mod model;

use actix_web::{middleware::Logger, App, HttpServer, Result};
use env_logger::Env;
use model::Model;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let model = Model::new().await.unwrap();

    HttpServer::new(move || {
        App::new()
            .data(model.clone())
            .wrap(Logger::default())
            .wrap(middleware::authentication())
            .configure(controller::config)
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}

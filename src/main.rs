#![feature(trait_alias)]
#![feature(async_closure)]
#![cfg_attr(test, feature(type_alias_impl_trait), allow(warnings), feature(impl_trait_in_bindings))]

extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate futures_util;
extern crate log;
extern crate clap;
extern crate actix_web_httpauth;
extern crate smtp;

mod controller;
mod middleware;
mod model;
mod service;
#[allow(dead_code)]
mod utils;

#[cfg(test)]
mod test;

use std::time::Duration;

use actix_web::{App, HttpServer, dev::Server, middleware::Logger};
use env_logger::Env;
use model::Model;
use service::EmailNotifyService;

async fn start_server(addr: &str) -> std::io::Result<Server> {
    let matches = clap::App::new("Sar Push Service")
        .version("0.1")
        .author("SardineFish")
        .about("Notify push service.")
        .arg("--init 'Init service database'")
        .get_matches();

    let model = Model::new().await.unwrap();
    let notify_service = EmailNotifyService::new(model.clone(), Duration::from_secs(5));

    if matches.is_present("init") {
        model.init_db().await.unwrap();
    }

    let server = HttpServer::new(move || {
        App::new()
            .data(model.clone())
            .data(notify_service.clone())
            .wrap(middleware::authentication())
            .wrap(middleware::error_formatter())
            .wrap(Logger::default())
            .configure(controller::config)
    })
    .bind(addr)?
    .run();
    Ok(server)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
 
    start_server("localhost:5000").await?.await
}

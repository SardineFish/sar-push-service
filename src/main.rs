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

async fn start_server(db_addr: &str, db_name: &str, listen_addr: &str) -> std::io::Result<Server> {

    let model = Model::new(db_addr, db_name).await.unwrap();
    let notify_service = EmailNotifyService::new(model.clone(), Duration::from_secs(5));

    let server = HttpServer::new(move || {
        App::new()
            .data(model.clone())
            .data(notify_service.clone())
            .wrap(middleware::authentication())
            .wrap(middleware::error_formatter())
            .wrap(Logger::default())
            .configure(controller::config)
    })
    .bind(listen_addr)?
    .run();
    Ok(server)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();
    
    let matches = clap::App::new("Sar Push Service")
        .version("0.1")
        .author("SardineFish")
        .about("Notify push service.")
        .arg("--init 'Init service database'")
        .arg("-l, --listen=[LOCAL_ADDR] 'Specific the local address [<host>:<port>] on which HTTP server will listen'")
        .arg("--db-addr=[DB_ADDR] 'Specific address of the mongodb service'")
        .arg("--db-name=[DB_NAME] 'Specific the mongodb db name to use for this service'")
        .get_matches();

    let local_addr = matches.value_of("listen").unwrap_or("localhost:5000");
    let db_name = matches.value_of("db-name").unwrap_or("sar-notify");
    let db_addr = matches.value_of("db-addr").unwrap_or("mongodb://mongo");
        
    if matches.is_present("init") {
        let model = Model::new(db_addr, db_name).await.unwrap();
        model.init_db().await.unwrap();
        log::info!("Service init successfully.");
        std::process::exit(0);
    }

    log::info!("Server listen on '{}'", local_addr);
    log::info!("Mongodb connect to '{}'", db_addr);
    log::info!("Use db '{}'", db_name);

 
    start_server(db_addr, db_name, local_addr).await?.await
}

#![feature(trait_alias)]

extern crate actix_web;
extern crate env_logger;
extern crate futures;
extern crate futures_util;
extern crate log;
extern crate clap;
extern crate actix_web_httpauth;

mod controller;
mod middleware;
mod model;
mod utils;

use actix_web::{middleware::Logger, App, HttpServer, Result};
use env_logger::Env;
use model::Model;
use clap::{Arg};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let matches = clap::App::new("Sar Push Service")
        .version("0.1")
        .author("SardineFish")
        .about("Notify push service.")
        .arg("--init 'Init service database'")
        .get_matches();
    
    
    let model = Model::new().await.unwrap();

    if matches.is_present("init") {
        model.init_db().await.unwrap();
    }

    HttpServer::new(move || {
        App::new()
            .data(model.clone())
            .wrap(middleware::authentication())
            .wrap(Logger::default())
            .configure(controller::config)
    })
    .bind("0.0.0.0:5000")?
    .run()
    .await
}

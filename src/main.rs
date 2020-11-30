extern crate actix_web;
extern crate futures_util;
extern crate futures;

mod model;
mod controller;
mod middleware;

use actix_web::{ HttpServer, App, web, get, HttpRequest, HttpResponse, Result };

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    HttpServer::new(|| App::new().configure(controller::config))
        .bind("0.0.0.0:5000")?
        .run()
        .await
}

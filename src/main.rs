extern crate actix_web;

mod model;
mod controller;

use actix_web::{ HttpServer, App, web, get, HttpRequest, HttpResponse, Result };

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    HttpServer::new(|| App::new().configure(controller::config))
        .bind("0.0.0.0:5000")?
        .run()
        .await
}

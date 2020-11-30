use actix_web::{ web, get, HttpRequest, HttpResponse, Result};
use crate::middleware;
use crate::model::Access;

#[get("/grant")]
pub async fn grant() -> Result<HttpResponse> {
    Ok(HttpResponse::Forbidden().finish())
}

pub fn config(cfg: &mut actix_web::web::ServiceConfig){
    cfg
        .service(grant);
}
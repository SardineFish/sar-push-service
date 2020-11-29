mod access;
mod notify;

use crate::model;
use actix_web::web;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/access").configure(access::config))
        .service(web::scope("/notify").configure(notify::config))
        .data(model::Model::new());
}
mod access;
mod notify;

use crate::model;
use crate::middleware;
use actix_web::web;
use middleware::FuncMiddleware;

use actix_web::{
    dev::ServiceRequest,dev::Service
};
use model::Access;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/access")
            .wrap(FuncMiddleware::from_func(middleware::access_check(Access::Admin)))
            .configure(access::config))
        .service(web::scope("/notify").configure(notify::config))
        .data(model::Model::new());
}
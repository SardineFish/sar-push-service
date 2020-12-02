mod access;
mod notify;

use crate::middleware;
use crate::model;
use actix_web::web;
use middleware::{FuncMiddleware, ServiceGuard};

use actix_web::{dev::Service, dev::ServiceRequest};
use model::{Access, NotifyProfile};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/access")
            .guard(ServiceGuard::<NotifyProfile>::new())
            .configure(access::config),
    )
    .service(web::scope("/notify").configure(notify::config))
    .data(model::Model::new());
}

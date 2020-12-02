mod access;
mod notify;
mod responder;
mod extractor;

use crate::middleware;
use crate::model;
use actix_web::web;
use middleware::{FuncMiddleware, ServiceGuard, service_guard};

use actix_web::{dev::Service, dev::ServiceRequest};
use model::{Access, AccessManagerProfile, NotifyProfile};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/access")
            .wrap(service_guard::<AccessManagerProfile, _, _>())
            .configure(access::config),
    )
    .service(web::scope("/notify").configure(notify::config))
    .data(model::Model::new());
}

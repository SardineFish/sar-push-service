mod access;
mod notify;
mod responder;
mod extractor;
mod service;
mod access_check;

use crate::middleware;
use crate::model;
use actix_web::web;
use middleware::{FuncMiddleware, ServiceGuard, service_guard};

use actix_web::{dev::Service, dev::ServiceRequest};
use model::{Access, AccessManagerProfile, NotifyProfile, ServiceManagerProfile};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/access")
            .wrap(service_guard::<AccessManagerProfile, _, _>())
            .configure(access::config),
    )
        .service(
            web::scope("/service")
                .wrap(service_guard::<ServiceManagerProfile, _, _>())
                .configure(service::config),
        )
    .data(model::Model::new());
}

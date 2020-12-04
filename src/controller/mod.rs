mod access;
mod notify;
mod extractor;
mod service;
mod access_check;

use crate::middleware;
use crate::model;
use actix_web::web;
use middleware::{service_guard};

use model::{AccessManagerProfile, ServiceManagerProfile};

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

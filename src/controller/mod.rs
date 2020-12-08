mod access;
mod access_check;
mod extractor;
mod notify;
mod service;

use crate::middleware;
use crate::model;
use actix_web::web;
use middleware::service_guard;

use model::{AccessManagerProfile, NotifyProfile, ServiceManagerProfile};

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
    .service(
        web::scope("/notify")
            .wrap(service_guard::<NotifyProfile, _, _>())
            .configure(notify::config),
    );
}

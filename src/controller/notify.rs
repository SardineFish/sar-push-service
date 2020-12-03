use actix_web::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Params {
    id: String,
}

#[get("/queue")]
async fn queue(params: web::Query<Params>, body: String) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(queue);
}
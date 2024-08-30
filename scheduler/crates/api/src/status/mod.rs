use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::get_service_health::*;

async fn status() -> HttpResponse {
    HttpResponse::Ok().json(APIResponse {
        message: "Ok!\r\n".into(),
    })
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthcheck", web::get().to(status));
}

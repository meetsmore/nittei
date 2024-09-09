use actix_web::{web, HttpResponse};
use nettu_scheduler_api_structs::get_service_health::*;
use nettu_scheduler_infra::NettuContext;

async fn status(ctx: web::Data<NettuContext>) -> HttpResponse {
    match ctx.repos.status.check_connection().await {
        Ok(_) => HttpResponse::Ok().json(APIResponse {
            message: "Ok!\r\n".into(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(APIResponse {
            message: "Internal Server Error".into(),
        }),
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/healthcheck", web::get().to(status));
}

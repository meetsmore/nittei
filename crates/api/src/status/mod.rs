use actix_web::{web, HttpResponse};
use nittei_api_structs::get_service_health::*;
use nittei_infra::NitteiContext;

async fn status(ctx: web::Data<NitteiContext>) -> HttpResponse {
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

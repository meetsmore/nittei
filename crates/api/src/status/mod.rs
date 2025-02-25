use actix_web::{HttpResponse, web};
use nittei_api_structs::get_service_health::*;
use nittei_infra::NitteiContext;

/// Get the status of the service
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

/// Configure the routes for the status module
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Get the health status of the service
    cfg.route("/healthcheck", web::get().to(status));
}

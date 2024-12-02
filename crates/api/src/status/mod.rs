use actix_web::{web, HttpResponse};
use nittei_api_structs::get_service_health::*;
use nittei_infra::NitteiContext;

use crate::ServerSharedState;

/// Get the status of the service
async fn status(
    ctx: web::Data<NitteiContext>,
    shared_state: web::Data<ServerSharedState>,
) -> HttpResponse {
    let is_shutting_down = shared_state.is_shutting_down.lock().await;

    if *is_shutting_down {
        return HttpResponse::ServiceUnavailable().json(APIResponse {
            message: "Service is shutting down".into(),
        });
    }

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

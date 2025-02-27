use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use nittei_api_structs::get_service_health::*;
use nittei_infra::NitteiContext;

/// Get the status of the service
async fn status(State(ctx): State<NitteiContext>) -> (StatusCode, Json<APIResponse>) {
    match ctx.repos.status.check_connection().await {
        Ok(_) => (
            StatusCode::OK,
            Json(APIResponse {
                message: "Ok!\r\n".into(),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(APIResponse {
                message: "Internal Server Error".into(),
            }),
        ),
    }
}

/// Configure the routes for the status module
pub fn configure_routes(router: &mut Router) {
    // Get the health status of the service
    router.route("/healthcheck", get(status));
}

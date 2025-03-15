use axum::{Extension, Json, Router, extract::State, http::StatusCode, routing::get};
use nittei_api_structs::get_service_health::*;
use nittei_infra::NitteiContext;

/// Get the status of the service
async fn status(Extension(ctx): Extension<NitteiContext>) -> (StatusCode, Json<APIResponse>) {
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
pub fn configure_routes() -> Router {
    Router::new()
        // Get the health status of the service
        .route("/healthcheck", get(status))
}

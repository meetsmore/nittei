use axum::{extract::Extension, http::StatusCode, response::IntoResponse, routing::get, Router};
use nittei_api_structs::get_service_health::*;
use nittei_infra::NitteiContext;
use std::sync::Arc;

use crate::ServerSharedState;

/// Get the status of the service
async fn status(
    Extension(ctx): Extension<Arc<NitteiContext>>,
    Extension(shared_state): Extension<Arc<ServerSharedState>>,
) -> impl IntoResponse {
    let is_shutting_down = shared_state.is_shutting_down.lock().await;

    if *is_shutting_down {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            axum::Json(APIResponse {
                message: "Service is shutting down".into(),
            }),
        );
    }

    match ctx.repos.status.check_connection().await {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(APIResponse {
                message: "Ok!\r\n".into(),
            }),
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(APIResponse {
                message: "Internal Server Error".into(),
            }),
        ),
    }
}

/// Configure the routes for the status module
pub fn configure_routes() -> Router {
    Router::new().route("/healthcheck", get(status))
}

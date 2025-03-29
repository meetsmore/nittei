mod create_schedule;
mod delete_schedule;
mod get_schedule;
mod get_schedules_by_meta;
mod update_schedule;

use axum::routing::{delete, get, post, put};
use create_schedule::{create_schedule_admin_controller, create_schedule_controller};
use delete_schedule::{delete_schedule_admin_controller, delete_schedule_controller};
use get_schedule::{get_schedule_admin_controller, get_schedule_controller};
use get_schedules_by_meta::get_schedules_by_meta_controller;
use update_schedule::{update_schedule_admin_controller, update_schedule_controller};
use utoipa_axum::router::OpenApiRouter;

pub fn configure_routes() -> OpenApiRouter {
    OpenApiRouter::new()
        .route("/schedule", post(create_schedule_controller))
        .route(
            "/user/{user_id}/schedule",
            post(create_schedule_admin_controller),
        )
        .route("/schedule/meta", get(get_schedules_by_meta_controller))
        .route("/schedule/{schedule_id}", get(get_schedule_controller))
        .route(
            "/user/schedule/{schedule_id}",
            get(get_schedule_admin_controller),
        )
        .route(
            "/schedule/{schedule_id}",
            delete(delete_schedule_controller),
        )
        .route(
            "/user/schedule/{schedule_id}",
            delete(delete_schedule_admin_controller),
        )
        .route("/schedule/{schedule_id}", put(update_schedule_controller))
        .route(
            "/user/schedule/{schedule_id}",
            put(update_schedule_admin_controller),
        )
}

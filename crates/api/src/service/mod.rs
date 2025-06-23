mod add_busy_calendar;
mod add_user_to_service;
mod create_service;
mod create_service_event_intend;
mod delete_service;
mod get_service;
mod get_service_bookingslots;
mod get_services_by_meta;
mod remove_busy_calendar;
mod remove_service_event_intend;
mod remove_user_from_service;
mod update_service;
mod update_service_user;

use add_busy_calendar::add_busy_calendar_controller;
use add_user_to_service::add_user_to_service_controller;
use axum::routing::{delete, get, post, put};
use create_service::create_service_controller;
use create_service_event_intend::create_service_event_intend_controller;
use delete_service::delete_service_controller;
use get_service::get_service_controller;
use get_service_bookingslots::get_service_bookingslots_controller;
use get_services_by_meta::get_services_by_meta_controller;
use remove_busy_calendar::remove_busy_calendar_controller;
use remove_service_event_intend::remove_service_event_intend_controller;
use remove_user_from_service::remove_user_from_service_controller;
use update_service::update_service_controller;
use update_service_user::update_service_user_controller;
use utoipa_axum::router::OpenApiRouter;

use crate::shared::auth;

pub fn configure_routes() -> OpenApiRouter {
    OpenApiRouter::new()
        .route("/service", post(create_service_controller))
        .route("/service/meta", get(get_services_by_meta_controller))
        .route("/service/{service_id}", get(get_service_controller))
        .route("/service/{service_id}", put(update_service_controller))
        .route("/service/{service_id}", delete(delete_service_controller))
        .route(
            "/service/{service_id}/users",
            post(add_user_to_service_controller),
        )
        .route(
            "/service/{service_id}/users/{user_id}",
            delete(remove_user_from_service_controller),
        )
        .route(
            "/service/{service_id}/users/{user_id}",
            put(update_service_user_controller),
        )
        .route(
            "/service/{service_id}/users/{user_id}/busy",
            put(add_busy_calendar_controller),
        )
        .route(
            "/service/{service_id}/users/{user_id}/busy",
            delete(remove_busy_calendar_controller),
        )
        .route(
            "/service/{service_id}/booking",
            get(get_service_bookingslots_controller),
        )
        .route(
            "/service/{service_id}/booking-intend",
            post(create_service_event_intend_controller),
        )
        .route(
            "/service/{service_id}/booking-intend",
            delete(remove_service_event_intend_controller),
        )
        .route_layer(axum::middleware::from_fn(auth::protect_admin_route_middleware))
}

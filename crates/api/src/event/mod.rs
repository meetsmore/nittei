pub mod create_event;
pub mod delete_event;
pub mod delete_many_events;
pub mod get_event;
pub mod get_event_by_external_id;
pub mod get_event_instances;
pub mod get_events_by_calendars;
pub mod get_events_by_meta;
pub mod get_events_for_users_in_time_range;
pub mod get_upcoming_reminders;
pub mod search_events;
pub mod subscribers;
pub mod sync_event_reminders;
pub mod update_event;

use axum::routing::{delete, get, post, put};
use create_event::{create_event_admin_controller, create_event_controller};
use delete_event::{delete_event_admin_controller, delete_event_controller};
use delete_many_events::delete_many_events_admin_controller;
use get_event::{get_event_admin_controller, get_event_controller};
use get_event_instances::{get_event_instances_admin_controller, get_event_instances_controller};
use get_events_by_meta::get_events_by_meta_controller;
use search_events::search_events_controller;
use update_event::{update_event_admin_controller, update_event_controller};
use utoipa_axum::router::OpenApiRouter;

use crate::shared::auth;

// Configure the routes for the event module
pub fn configure_routes() -> OpenApiRouter {
    let admin_router = OpenApiRouter::new()
        // Create an event for a user (admin route)
        .route(
            "/user/{user_id}/events",
            post(create_event_admin_controller),
        )
        // Search events
        // /!\ This is a POST route
        .route("/events/search", post(search_events_controller))
        // Get events by metadata
        .route("/events/meta", get(get_events_by_meta_controller))
        // Get events of multiple users during a time range
        .route(
            "/events/timespan",
            post(get_events_for_users_in_time_range::get_events_for_users_in_time_range_controller),
        )
        // Get events by calendars
        .route(
            "/user/{user_id}/events",
            get(get_events_by_calendars::get_events_by_calendars_controller),
        )
        // Get a specific event by external id
        .route(
            "/user/events/external_id/{external_id}",
            get(get_event_by_external_id::get_event_by_external_id_admin_controller),
        )
        // Get a specific event by uid (admin route)
        .route("/user/events/{event_id}", get(get_event_admin_controller))
        // Update an event by uid (admin route)
        .route(
            "/user/events/{event_id}",
            put(update_event_admin_controller),
        )
        // Delete an event by uid (admin route)
        .route(
            "/user/events/{event_id}",
            delete(delete_event_admin_controller),
        )
        // Get event instances (admin route)
        .route(
            "/user/events/{event_id}/instances",
            get(get_event_instances_admin_controller),
        )
        // Admin delete many events
        .route(
            "/user/events/delete_many",
            post(delete_many_events_admin_controller),
        )
        .route_layer(axum::middleware::from_fn(auth::protect_admin_route));

    let user_router = OpenApiRouter::new()
        // Create an event
        .route("/events", post(create_event_controller))
        // Get a specific event by uid
        .route("/events/{event_id}", get(get_event_controller))
        // Update an event by uid
        .route("/events/{event_id}", put(update_event_controller))
        // Delete an event by uid
        .route("/events/{event_id}", delete(delete_event_controller))
        // Get event instances
        .route(
            "/events/{event_id}/instances",
            get(get_event_instances_controller),
        );

    OpenApiRouter::new().merge(admin_router).merge(user_router)
}

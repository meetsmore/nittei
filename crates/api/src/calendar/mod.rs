pub mod add_sync_calendar;
pub mod create_calendar;
pub mod delete_calendar;
pub mod get_calendar;
pub mod get_calendar_events;
pub mod get_calendars;
pub mod get_calendars_by_meta;
pub mod get_google_calendars;
pub mod get_outlook_calendars;
pub mod remove_sync_calendar;
pub mod update_calendar;

use add_sync_calendar::add_sync_calendar_admin_controller;
use axum::routing::{delete, get, post, put};
use create_calendar::{create_calendar_admin_controller, create_calendar_controller};
use delete_calendar::{delete_calendar_admin_controller, delete_calendar_controller};
use get_calendar::{get_calendar_admin_controller, get_calendar_controller};
use get_calendar_events::{get_calendar_events_admin_controller, get_calendar_events_controller};
use get_calendars_by_meta::get_calendars_by_meta_controller;
use get_google_calendars::{
    get_google_calendars_admin_controller,
    get_google_calendars_controller,
};
use get_outlook_calendars::{
    get_outlook_calendars_admin_controller,
    get_outlook_calendars_controller,
};
use remove_sync_calendar::remove_sync_calendar_admin_controller;
use update_calendar::{update_calendar_admin_controller, update_calendar_controller};
use utoipa_axum::router::OpenApiRouter;

use crate::shared::auth;

/// Configure the routes for the calendar module
pub fn configure_routes() -> OpenApiRouter {
    let admin_router = OpenApiRouter::new()
        // Create a calendar for a user (admin route)
        .route(
            "/user/{user_id}/calendar",
            post(create_calendar_admin_controller),
        )
        // List calendars for a user (admin route)
        .route(
            "/user/{user_id}/calendar",
            get(get_calendars::get_calendars_admin_controller),
        )
        // List calendars by metadata (admin route)
        .route("/calendar/meta", get(get_calendars_by_meta_controller))
        // Get a specific calendar by uid for a user (admin route)
        .route(
            "/user/calendar/{calendar_id}",
            get(get_calendar_admin_controller),
        )
        // Delete a calendar by uid for a user (admin route)
        .route(
            "/user/calendar/{calendar_id}",
            delete(delete_calendar_admin_controller),
        )
        // Update a calendar by uid for a user (admin route)
        .route(
            "/user/calendar/{calendar_id}",
            put(update_calendar_admin_controller),
        )
        // Get events for a calendar for a user (admin route)
        .route(
            "/user/calendar/{calendar_id}/events",
            get(get_calendar_events_admin_controller),
        )
        .route(
            "/user/{user_id}/calendar/provider/google",
            get(get_google_calendars_admin_controller),
        )
        .route(
            "/user/{user_id}/calendar/provider/outlook",
            get(get_outlook_calendars_admin_controller),
        )
        .route(
            "/user/{user_id}/calendar/sync",
            put(add_sync_calendar_admin_controller),
        )
        .route(
            "/user/{user_id}/calendar/sync",
            delete(remove_sync_calendar_admin_controller),
        )
        .route_layer(axum::middleware::from_fn(auth::protect_admin_route));

    let user_router = OpenApiRouter::new()
        // Create a calendar
        .route("/calendar", post(create_calendar_controller))
        // List calendars
        .route("/calendar", get(get_calendars::get_calendars_controller))
        // Get a specific calendar by uid
        .route("/calendar/{calendar_id}", get(get_calendar_controller))
        // Delete a calendar by uid
        .route(
            "/calendar/{calendar_id}",
            delete(delete_calendar_controller),
        )
        // Update a calendar by uid
        .route("/calendar/{calendar_id}", put(update_calendar_controller))
        // Get events for a calendar
        .route(
            "/calendar/{calendar_id}/events",
            get(get_calendar_events_controller),
        )
        // Calendar providers
        .route(
            "/calendar/provider/google",
            get(get_google_calendars_controller),
        )
        .route(
            "/calendar/provider/outlook",
            get(get_outlook_calendars_controller),
        );

    OpenApiRouter::new().merge(admin_router).merge(user_router)
}

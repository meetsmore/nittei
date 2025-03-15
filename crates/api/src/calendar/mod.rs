mod add_sync_calendar;
mod create_calendar;
mod delete_calendar;
mod get_calendar;
mod get_calendar_events;
mod get_calendars;
mod get_calendars_by_meta;
mod get_google_calendars;
mod get_outlook_calendars;
mod remove_sync_calendar;
mod update_calendar;

use add_sync_calendar::add_sync_calendar_admin_controller;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
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

/// Configure the routes for the calendar module
pub fn configure_routes() -> Router {
    Router::new()
        // Create a calendar
        .route("/calendar", post(create_calendar_controller))
        // Create a calendar for a user (admin route)
        .route(
            "/user/{user_id}/calendar",
            post(create_calendar_admin_controller),
        )
        // List calendars
        .route("/calendar", get(get_calendars::get_calendars_controller))
        // List calendars for a user (admin route)
        .route(
            "/user/{user_id}/calendar",
            get(get_calendars::get_calendars_admin_controller),
        )
        // List calendars by metadata
        .route("/calendar/meta", get(get_calendars_by_meta_controller))
        // Get a specific calendar by uid
        .route("/calendar/{calendar_id}", get(get_calendar_controller))
        // Get a specific calendar by uid for a user (admin route)
        .route(
            "/user/calendar/{calendar_id}",
            get(get_calendar_admin_controller),
        )
        // Delete a calendar by uid
        .route(
            "/calendar/{calendar_id}",
            delete(delete_calendar_controller),
        )
        // Delete a calendar by uid for a user (admin route)
        .route(
            "/user/calendar/{calendar_id}",
            delete(delete_calendar_admin_controller),
        )
        // Update a calendar by uid
        .route("/calendar/{calendar_id}", put(update_calendar_controller))
        // Update a calendar by uid for a user (admin route)
        .route(
            "/user/calendar/{calendar_id}",
            put(update_calendar_admin_controller),
        )
        // Get events for a calendar
        .route(
            "/calendar/{calendar_id}/events",
            get(get_calendar_events_controller),
        )
        // Get events for a calendar for a user (admin route)
        .route(
            "/user/calendar/{calendar_id}/events",
            get(get_calendar_events_admin_controller),
        )
        // Calendar providers
        .route(
            "/calendar/provider/google",
            get(get_google_calendars_controller),
        )
        .route(
            "/user/{user_id}/calendar/provider/google",
            get(get_google_calendars_admin_controller),
        )
        .route(
            "/calendar/provider/outlook",
            get(get_outlook_calendars_controller),
        )
        .route(
            "/user/{user_id}/calendar/provider/outlook",
            get(get_outlook_calendars_admin_controller),
        )
        // cfg.route(
        //     "/calendar/sync/",
        //     web::put().to(add_sync_calendar_controller),
        // )
        .route(
            "/user/{user_id}/calendar/sync",
            put(add_sync_calendar_admin_controller),
        )
        // cfg.route(
        //     "/calendar/sync",
        //     web::delete().to(remove_sync_calendar_controller),
        // )
        .route(
            "/user/{user_id}/calendar/sync",
            delete(remove_sync_calendar_admin_controller),
        )
}

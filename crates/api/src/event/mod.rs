mod create_event;
mod delete_event;
mod delete_many_events;
mod get_event;
mod get_event_by_external_id;
mod get_event_instances;
mod get_events_by_calendars;
mod get_events_by_meta;
pub mod get_upcoming_reminders;
mod search_events;
mod subscribers;
pub mod sync_event_reminders;
mod update_event;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use create_event::{create_event_admin_controller, create_event_controller};
use delete_event::{delete_event_admin_controller, delete_event_controller};
use delete_many_events::delete_many_events_admin_controller;
use get_event::{get_event_admin_controller, get_event_controller};
use get_event_instances::{get_event_instances_admin_controller, get_event_instances_controller};
use get_events_by_meta::get_events_by_meta_controller;
use search_events::search_events_controller;
use update_event::{update_event_admin_controller, update_event_controller};

// Configure the routes for the event module
pub fn configure_routes(router: &mut Router) {
    // Create an event
    router.route("/events", post(create_event_controller));
    // Create an event for a user (admin route)
    router.route(
        "/user/{user_id}/events",
        post(create_event_admin_controller),
    );

    // Get events by calendars
    router.route(
        "/user/{user_id}/events",
        get(get_events_by_calendars::get_events_by_calendars_controller),
    );

    // Get events by metadata
    router.route("/events/meta", get(get_events_by_meta_controller));

    // Search events
    // /!\ This is a POST route
    router.route("/events/search", get(search_events_controller));

    // Get a specific event by external id
    router.route(
        "/user/events/external_id/{external_id}",
        get(get_event_by_external_id::get_event_by_external_id_admin_controller),
    );

    // Get a specific event by uid
    router.route("/events/{event_id}", get(get_event_controller));
    // Get a specific event by uid (admin route)
    router.route("/user/events/{event_id}", get(get_event_admin_controller));

    // Delete an event by uid
    router.route("/events/{event_id}", delete(delete_event_controller));
    // Delete an event by uid (admin route)
    router.route(
        "/user/events/{event_id}",
        delete(delete_event_admin_controller),
    );

    // Update an event by uid
    router.route("/events/{event_id}", put(update_event_controller));
    // Update an event by uid (admin route)
    router.route(
        "/user/events/{event_id}",
        put(update_event_admin_controller),
    );

    // Get event instances
    router.route(
        "/events/{event_id}/instances",
        get(get_event_instances_controller),
    );
    // Get event instances (admin route)
    router.route(
        "/user/events/{event_id}/instances",
        get(get_event_instances_admin_controller),
    );

    // Admin delete many events
    cfg.route(
        "/user/events/delete_many",
        web::post().to(delete_many_events_admin_controller),
    );
}

mod create_event;
mod delete_event;
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

use actix_web::web;
use create_event::{create_event_admin_controller, create_event_controller};
use delete_event::{delete_event_admin_controller, delete_event_controller};
use get_event::{get_event_admin_controller, get_event_controller};
use get_event_instances::{get_event_instances_admin_controller, get_event_instances_controller};
use get_events_by_meta::get_events_by_meta_controller;
use search_events::search_events_controller;
use update_event::{update_event_admin_controller, update_event_controller};

// Configure the routes for the event module
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Create an event
    cfg.route("/events", web::post().to(create_event_controller));
    // Create an event for a user (admin route)
    cfg.route(
        "/user/{user_id}/events",
        web::post().to(create_event_admin_controller),
    );

    // Get events by calendars
    cfg.route(
        "/user/{user_id}/events",
        web::get().to(get_events_by_calendars::get_events_by_calendars_controller),
    );

    // Get events by metadata
    cfg.route("/events/meta", web::get().to(get_events_by_meta_controller));

    // Search events
    // /!\ This is a POST route
    cfg.route("/events/search", web::post().to(search_events_controller));

    // Get a specific event by external id
    cfg.route(
        "/user/events/external_id/{external_id}",
        web::get().to(get_event_by_external_id::get_event_by_external_id_admin_controller),
    );

    // Get a specific event by uid
    cfg.route("/events/{event_id}", web::get().to(get_event_controller));
    // Get a specific event by uid (admin route)
    cfg.route(
        "/user/events/{event_id}",
        web::get().to(get_event_admin_controller),
    );

    // Delete an event by uid
    cfg.route(
        "/events/{event_id}",
        web::delete().to(delete_event_controller),
    );
    // Delete an event by uid (admin route)
    cfg.route(
        "/user/events/{event_id}",
        web::delete().to(delete_event_admin_controller),
    );

    // Update an event by uid
    cfg.route("/events/{event_id}", web::put().to(update_event_controller));
    // Update an event by uid (admin route)
    cfg.route(
        "/user/events/{event_id}",
        web::put().to(update_event_admin_controller),
    );

    // Get event instances
    cfg.route(
        "/events/{event_id}/instances",
        web::get().to(get_event_instances_controller),
    );
    // Get event instances (admin route)
    cfg.route(
        "/user/events/{event_id}/instances",
        web::get().to(get_event_instances_admin_controller),
    );
}

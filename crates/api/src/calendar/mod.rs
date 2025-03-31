use actix_web::web;

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
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Create a calendar
    cfg.route("/calendar", web::post().to(create_calendar_controller));
    // Create a calendar for a user (admin route)
    cfg.route(
        "/user/{user_id}/calendar",
        web::post().to(create_calendar_admin_controller),
    );

    // List calendars
    cfg.route(
        "/calendar",
        web::get().to(get_calendars::get_calendars_controller),
    );
    // List calendars for a user (admin route)
    cfg.route(
        "/user/{user_id}/calendar",
        web::get().to(get_calendars::get_calendars_admin_controller),
    );

    // List calendars by metadata
    cfg.route(
        "/calendar/meta",
        web::get().to(get_calendars_by_meta_controller),
    );

    // Get a specific calendar by uid
    cfg.route(
        "/calendar/{calendar_id}",
        web::get().to(get_calendar_controller),
    );
    // Get a specific calendar by uid for a user (admin route)
    cfg.route(
        "/user/calendar/{calendar_id}",
        web::get().to(get_calendar_admin_controller),
    );

    // Delete a calendar by uid
    cfg.route(
        "/calendar/{calendar_id}",
        web::delete().to(delete_calendar_controller),
    );
    // Delete a calendar by uid for a user (admin route)
    cfg.route(
        "/user/calendar/{calendar_id}",
        web::delete().to(delete_calendar_admin_controller),
    );

    // Update a calendar by uid
    cfg.route(
        "/calendar/{calendar_id}",
        web::put().to(update_calendar_controller),
    );
    // Update a calendar by uid for a user (admin route)
    cfg.route(
        "/user/calendar/{calendar_id}",
        web::put().to(update_calendar_admin_controller),
    );

    // Get events for a calendar
    cfg.route(
        "/calendar/{calendar_id}/events",
        web::get().to(get_calendar_events_controller),
    );
    // Get events for a calendar for a user (admin route)
    cfg.route(
        "/user/calendar/{calendar_id}/events",
        web::get().to(get_calendar_events_admin_controller),
    );

    // Calendar providers
    cfg.route(
        "/calendar/provider/google",
        web::get().to(get_google_calendars_controller),
    );
    cfg.route(
        "/user/{user_id}/calendar/provider/google",
        web::get().to(get_google_calendars_admin_controller),
    );
    cfg.route(
        "/calendar/provider/outlook",
        web::get().to(get_outlook_calendars_controller),
    );
    cfg.route(
        "/user/{user_id}/calendar/provider/outlook",
        web::get().to(get_outlook_calendars_admin_controller),
    );
    // cfg.route(
    //     "/calendar/sync/",
    //     web::put().to(add_sync_calendar_controller),
    // );
    cfg.route(
        "/user/{user_id}/calendar/sync",
        web::put().to(add_sync_calendar_admin_controller),
    );
    // cfg.route(
    //     "/calendar/sync",
    //     web::delete().to(remove_sync_calendar_controller),
    // );
    cfg.route(
        "/user/{user_id}/calendar/sync",
        web::delete().to(remove_sync_calendar_admin_controller),
    );
}

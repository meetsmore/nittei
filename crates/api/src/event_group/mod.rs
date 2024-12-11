mod create_event_group;

use actix_web::web;
use create_event_group::create_event_group_admin_controller;

// Configure the routes for the event_group module
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Create an event for a user (admin route)
    cfg.route(
        "/user/{user_id}/event_group",
        web::post().to(create_event_group_admin_controller),
    );
}

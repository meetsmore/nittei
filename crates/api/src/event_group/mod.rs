mod create_event_group;
mod delete_event_group;
mod get_event_group;
mod get_event_group_by_external_id;
mod update_event_group;

use actix_web::web;
use create_event_group::create_event_group_admin_controller;
use delete_event_group::delete_event_group_admin_controller;
use get_event_group::get_event_group_admin_controller;
use get_event_group_by_external_id::get_event_group_by_external_id_admin_controller;
use update_event_group::update_event_group_admin_controller;

// Configure the routes for the event_group module
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Create an event group for a user (admin route)
    cfg.route(
        "/user/{user_id}/event_groups",
        web::post().to(create_event_group_admin_controller),
    );

    // Get a specific event group by external id
    cfg.route(
        "/user/event_groups/external_id/{external_id}",
        web::get().to(get_event_group_by_external_id_admin_controller),
    );

    // Get a specific event group by uid (admin route)
    cfg.route(
        "/user/event_groups/{event_group_id}",
        web::get().to(get_event_group_admin_controller),
    );

    // Update an event group by uid (admin route)
    cfg.route(
        "/user/event_groups/{event_group_id}",
        web::put().to(update_event_group_admin_controller),
    );

    // Delete an event group by uid (admin route)
    cfg.route(
        "/user/event_groups/{event_group_id}",
        web::delete().to(delete_event_group_admin_controller),
    );
}

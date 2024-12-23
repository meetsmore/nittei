mod create_event_group;
mod delete_event_group;
mod get_event_group;
mod get_event_group_by_external_id;
mod update_event_group;

use axum::{routing::get, routing::post, routing::put, routing::delete, Router};
use create_event_group::create_event_group_admin_controller;
use delete_event_group::delete_event_group_admin_controller;
use get_event_group::get_event_group_admin_controller;
use get_event_group_by_external_id::get_event_group_by_external_id_admin_controller;
use update_event_group::update_event_group_admin_controller;

// Configure the routes for the event_group module
pub fn configure_routes() -> Router {
    Router::new()
        // Create an event group for a user (admin route)
        .route("/user/:user_id/event_groups", post(create_event_group_admin_controller))
        // Get a specific event group by external id
        .route("/user/event_groups/external_id/:external_id", get(get_event_group_by_external_id_admin_controller))
        // Get a specific event group by uid (admin route)
        .route("/user/event_groups/:event_group_id", get(get_event_group_admin_controller))
        // Update an event group by uid (admin route)
        .route("/user/event_groups/:event_group_id", put(update_event_group_admin_controller))
        // Delete an event group by uid (admin route)
        .route("/user/event_groups/:event_group_id", delete(delete_event_group_admin_controller))
}

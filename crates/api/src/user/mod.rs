pub mod create_user;
mod delete_user;
mod get_me;
mod get_multiple_users_freebusy;
mod get_user;
mod get_user_by_external_id;
mod get_user_freebusy;
mod get_users_by_meta;
mod oauth_integration;
mod remove_integration;
mod update_user;

use axum::{
    routing::{get, post, put, delete},
    Router,
};
use create_user::create_user_controller;
use delete_user::delete_user_controller;
use get_me::get_me_controller;
use get_multiple_users_freebusy::get_multiple_freebusy_controller;
use get_user::get_user_controller;
use get_user_by_external_id::get_user_by_external_id_controller;
use get_user_freebusy::get_freebusy_controller;
pub use get_user_freebusy::parse_vec_query_value;
use get_users_by_meta::get_users_by_meta_controller;
use oauth_integration::*;
use remove_integration::{remove_integration_admin_controller, remove_integration_controller};
use update_user::update_user_controller;

// Configure the routes for the user module
pub fn configure_routes() -> Router {
    Router::new()
        // Create a new user
        .route("/user", post(create_user_controller))
        // Get the current user
        .route("/me", get(get_me_controller))
        // Get users by metadata
        .route("/user/meta", get(get_users_by_meta_controller))
        // Get user by external_id
        .route(
            "/user/external_id/:external_id",
            get(get_user_by_external_id_controller),
        )
        // Get freebusy for multiple users
        // This is a POST route !
        .route(
            "/user/freebusy",
            post(get_multiple_freebusy_controller),
        )
        // Get a specific user by id
        .route("/user/:user_id", get(get_user_controller))
        // Update a specific user by id
        .route("/user/:user_id", put(update_user_controller))
        // Delete a specific user by id
        .route("/user/:user_id", delete(delete_user_controller))
        // Get freebusy for a specific user
        .route(
            "/user/:user_id/freebusy",
            get(get_freebusy_controller),
        )
        // Oauth
        .route("/me/oauth", post(oauth_integration_controller))
        .route(
            "/me/oauth/:provider",
            delete(remove_integration_controller),
        )
        .route(
            "/user/:user_id/oauth",
            post(oauth_integration_admin_controller),
        )
        .route(
            "/user/:user_id/oauth/:provider",
            delete(remove_integration_admin_controller),
        )
}

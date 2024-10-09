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

use actix_web::web;
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
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Create a new user
    cfg.route("/user", web::post().to(create_user_controller));

    // Get the current user
    cfg.route("/me", web::get().to(get_me_controller));

    // Get users by metadata
    cfg.route("/user/meta", web::get().to(get_users_by_meta_controller));

    // Get user by external_id
    cfg.route(
        "/user/external_id/{external_id}",
        web::get().to(get_user_by_external_id_controller),
    );

    // Get freebusy for multiple users
    // This is a POST route !
    cfg.route(
        "/user/freebusy",
        web::post().to(get_multiple_freebusy_controller),
    );

    // Get a specific user by id
    cfg.route("/user/{user_id}", web::get().to(get_user_controller));

    // Update a specific user by id
    cfg.route("/user/{user_id}", web::put().to(update_user_controller));

    // Delete a specific user by id
    cfg.route("/user/{user_id}", web::delete().to(delete_user_controller));

    // Get freebusy for a specific user
    cfg.route(
        "/user/{user_id}/freebusy",
        web::get().to(get_freebusy_controller),
    );

    // Oauth
    cfg.route("/me/oauth", web::post().to(oauth_integration_controller));
    cfg.route(
        "/me/oauth/{provider}",
        web::delete().to(remove_integration_controller),
    );
    cfg.route(
        "/user/{user_id}/oauth",
        web::post().to(oauth_integration_admin_controller),
    );
    cfg.route(
        "/user/{user_id}/oauth/{provider}",
        web::delete().to(remove_integration_admin_controller),
    );
}

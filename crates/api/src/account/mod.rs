pub mod account_search_events;
pub mod add_account_integration;
pub mod create_account;
pub mod delete_account_webhook;
pub mod get_account;
pub mod remove_account_integration;
pub mod set_account_pub_key;
pub mod set_account_webhook;

use account_search_events::account_search_events_controller;
use actix_web::web;
use add_account_integration::add_account_integration_controller;
use create_account::create_account_controller;
use delete_account_webhook::delete_account_webhook_controller;
use get_account::get_account_controller;
use remove_account_integration::remove_account_integration_controller;
use set_account_pub_key::set_account_pub_key_controller;
use set_account_webhook::set_account_webhook_controller;

/// Configure the routes for the account module
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    // Create a new account
    cfg.route("/account", web::post().to(create_account_controller));

    // Get the account details
    cfg.route("/account", web::get().to(get_account_controller));

    // Set the public key for the account
    cfg.route(
        "/account/pubkey",
        web::put().to(set_account_pub_key_controller),
    );

    // Set the webhook for the account
    cfg.route(
        "/account/webhook",
        web::put().to(set_account_webhook_controller),
    );

    // Delete the webhook for the account
    cfg.route(
        "/account/webhook",
        web::delete().to(delete_account_webhook_controller),
    );

    // Add an integration for the account
    cfg.route(
        "/account/integration",
        web::put().to(add_account_integration_controller),
    );

    // Remove an integration for the account
    cfg.route(
        "/account/integration/{provider}",
        web::delete().to(remove_account_integration_controller),
    );

    // Search events across all users for the account
    cfg.route(
        "/account/events/search",
        web::post().to(account_search_events_controller),
    );
}

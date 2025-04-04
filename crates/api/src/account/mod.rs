pub mod account_search_events;
pub mod add_account_integration;
pub mod create_account;
pub mod delete_account_webhook;
pub mod get_account;
pub mod remove_account_integration;
pub mod set_account_pub_key;
pub mod set_account_webhook;

use account_search_events::account_search_events_controller;
use add_account_integration::add_account_integration_controller;
use axum::routing::{delete, get, post, put};
use create_account::create_account_controller;
use delete_account_webhook::delete_account_webhook_controller;
use get_account::get_account_controller;
use remove_account_integration::remove_account_integration_controller;
use set_account_pub_key::set_account_pub_key_controller;
use set_account_webhook::set_account_webhook_controller;
use utoipa_axum::router::OpenApiRouter;

/// Configure the routes for the account module
pub fn configure_routes() -> OpenApiRouter {
    OpenApiRouter::new()
        // Create a new account
        .route("/account", post(create_account_controller))
        // Get the account details
        .route("/account", get(get_account_controller))
        // Set the public key for the account
        .route("/account/pubkey", put(set_account_pub_key_controller))
        // Set the webhook for the account
        .route("/account/webhook", put(set_account_webhook_controller))
        // Delete the webhook for the account
        .route(
            "/account/webhook",
            delete(delete_account_webhook_controller),
        )
        // Add an integration for the account
        .route(
            "/account/integration",
            put(add_account_integration_controller),
        )
        // Remove an integration for the account
        .route(
            "/account/integration/{provider}",
            delete(remove_account_integration_controller),
        )
        // Search events across all users for the account
        .route(
            "/account/events/search",
            post(account_search_events_controller),
        )
}

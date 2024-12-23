mod add_account_integration;
mod create_account;
mod delete_account_webhook;
mod get_account;
mod remove_account_integration;
mod set_account_pub_key;
mod set_account_webhook;

use axum::{routing::get, routing::post, routing::put, routing::delete, Router};
use add_account_integration::add_account_integration_controller;
use create_account::create_account_controller;
use delete_account_webhook::delete_account_webhook_controller;
use get_account::get_account_controller;
use remove_account_integration::remove_account_integration_controller;
use set_account_pub_key::set_account_pub_key_controller;
use set_account_webhook::set_account_webhook_controller;

/// Configure the routes for the account module
pub fn configure_routes() -> Router {
    Router::new()
        // Create a new account
        .route("/account", post(create_account_controller))
        // Get the account details
        .route("/account", get(get_account_controller))
        // Set the public key for the account
        .route("/account/pubkey", put(set_account_pub_key_controller))
        // Set the webhook for the account
        .route("/account/webhook", put(set_account_webhook_controller))
        // Delete the webhook for the account
        .route("/account/webhook", delete(delete_account_webhook_controller))
        // Add an integration for the account
        .route("/account/integration", put(add_account_integration_controller))
        // Remove an integration for the account
        .route("/account/integration/:provider", delete(remove_account_integration_controller))
}

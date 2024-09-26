use nittei_domain::Account;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dtos::AccountDTO;

/// Account response object
#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountResponse {
    /// Account retrieved
    pub account: AccountDTO,
}

impl AccountResponse {
    pub fn new(account: Account) -> Self {
        Self {
            account: AccountDTO::new(&account),
        }
    }
}

pub mod create_account {
    use super::*;

    /// Request body for creating an account
    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateAccountRequestBody")]
    pub struct RequestBody {
        /// Code used for authentifying the request
        /// Creating accounts is an admin operation, so it requires a specific code
        pub code: String,
    }

    /// Response body for creating an account
    #[derive(Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateAccountResponseBody")]
    pub struct APIResponse {
        /// Account created
        pub account: AccountDTO,
        /// API Key that can be used for doing requests for this account
        pub secret_api_key: String,
    }

    impl APIResponse {
        pub fn new(account: Account) -> Self {
            Self {
                account: AccountDTO::new(&account),
                secret_api_key: account.secret_api_key,
            }
        }
    }
}

pub mod get_account {
    use super::*;

    pub type APIResponse = AccountResponse;
}

pub mod set_account_pub_key {
    use super::*;

    /// Request body for setting the public JWT key of an account
    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "SetAccountPubKeyRequestBody")]
    pub struct RequestBody {
        /// Public JWT key
        pub public_jwt_key: Option<String>,
    }

    pub type APIResponse = AccountResponse;
}

pub mod set_account_webhook {

    use super::*;

    /// Request body for setting the webhook of an account
    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "SetAccountWebhookRequestBody")]
    pub struct RequestBody {
        /// Webhook URL
        pub webhook_url: String,
    }

    pub type APIResponse = AccountResponse;
}

pub mod delete_account_webhook {
    use super::*;

    pub type APIResponse = AccountResponse;
}

pub mod add_account_integration {
    use nittei_domain::IntegrationProvider;

    use super::*;

    /// Request body for adding an integration to an account
    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "AddAccountIntegrationRequestBody")]
    pub struct RequestBody {
        /// Client ID of the integration
        pub client_id: String,
        // Client secret of the integration
        pub client_secret: String,
        // Redirect URI of the integration
        pub redirect_uri: String,
        /// Provider of the integration
        /// This is used to know which integration to use
        /// E.g. Google, Outlook, etc.
        pub provider: IntegrationProvider,
    }

    pub type APIResponse = String;
}

pub mod remove_account_integration {
    use nittei_domain::IntegrationProvider;

    use super::*;

    #[derive(Debug, Deserialize, Serialize)]
    pub struct PathParams {
        pub provider: IntegrationProvider,
    }

    pub type APIResponse = String;
}

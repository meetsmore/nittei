use nittei_domain::Account;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::dtos::AccountDTO;

#[derive(Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountResponse {
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

    #[derive(Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateAccountRequestBody")]
    pub struct RequestBody {
        pub code: String,
    }

    #[derive(Serialize, Deserialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "CreateAccountResponseBody")]
    pub struct APIResponse {
        pub account: AccountDTO,
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

    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "SetAccountPubKeyRequestBody")]
    pub struct RequestBody {
        pub public_jwt_key: Option<String>,
    }

    pub type APIResponse = AccountResponse;
}

pub mod set_account_webhook {

    use super::*;

    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "SetAccountWebhookRequestBody")]
    pub struct RequestBody {
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

    #[derive(Debug, Deserialize, Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export, rename = "AddAccountIntegrationRequestBody")]
    pub struct RequestBody {
        pub client_id: String,
        pub client_secret: String,
        pub redirect_uri: String,
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

use nittei_domain::{Account, AccountSettings, AccountWebhookSettings, PEMKey, ID};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Account - an account can have multiple users
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountDTO {
    /// The unique identifier for the account
    pub id: ID,
    /// Optional public key for JWT verification
    pub public_jwt_key: Option<PEMKey>,
    /// Account settings
    pub settings: AccountSettingsDTO,
}

impl AccountDTO {
    pub fn new(account: &Account) -> Self {
        Self {
            id: account.id.clone(),
            public_jwt_key: account.public_jwt_key.clone(),
            settings: AccountSettingsDTO::new(&account.settings),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountSettingsDTO {
    pub webhook: Option<AccountWebhookSettingsDTO>,
}

impl AccountSettingsDTO {
    pub fn new(settings: &AccountSettings) -> Self {
        let webhook_settings = settings
            .webhook
            .as_ref()
            .map(AccountWebhookSettingsDTO::new);

        Self {
            webhook: webhook_settings,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountWebhookSettingsDTO {
    pub url: String,
    pub key: String,
}

impl AccountWebhookSettingsDTO {
    pub fn new(settings: &AccountWebhookSettings) -> Self {
        Self {
            url: settings.url.clone(),
            key: settings.key.clone(),
        }
    }
}

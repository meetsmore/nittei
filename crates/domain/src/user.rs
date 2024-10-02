use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    shared::entity::{Entity, ID},
    Meta,
    Metadata,
};

#[derive(Debug, Clone, Default)]
pub struct User {
    pub id: ID,
    pub account_id: ID,
    pub metadata: Metadata,
}

impl User {
    pub fn new(account_id: ID, user_id: Option<ID>) -> Self {
        Self {
            account_id,
            id: user_id.unwrap_or_default(),
            ..Default::default()
        }
    }
}

impl Entity<ID> for User {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

impl Meta<ID> for User {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct UserIntegration {
    pub user_id: ID,
    pub account_id: ID,
    pub provider: IntegrationProvider,
    pub refresh_token: String,
    pub access_token: String,
    pub access_token_expires_ts: i64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum IntegrationProvider {
    #[default]
    Google,
    Outlook,
}

impl From<IntegrationProvider> for String {
    fn from(e: IntegrationProvider) -> Self {
        match e {
            IntegrationProvider::Google => "google".into(),
            IntegrationProvider::Outlook => "outlook".into(),
        }
    }
}

impl From<String> for IntegrationProvider {
    fn from(e: String) -> IntegrationProvider {
        match &e[..] {
            "google" => IntegrationProvider::Google,
            "outlook" => IntegrationProvider::Outlook,
            _ => unreachable!("Invalid provider"),
        }
    }
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct UserGoogleIntegrationData {
//     pub refresh_token: String,
//     pub access_token: String,
//     pub access_token_expires_ts: i64,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct UserOutlookIntegrationData {
//     pub refresh_token: String,
//     pub access_token: String,
//     pub access_token_expires_ts: i64,
// }

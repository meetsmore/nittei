use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ID;

/// Group of calendar events
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventGroup {
    pub id: ID,
    pub calendar_id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub parent_id: Option<String>,
    pub external_id: Option<String>,
    pub event_ids: Vec<ID>,
}

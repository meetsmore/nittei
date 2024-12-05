use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ID;

/// Group of calendar events
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EventGroup {
    /// Unique ID
    pub id: ID,

    /// Calendar ID to which the group belongs
    pub calendar_id: ID,

    /// User ID
    pub user_id: ID,

    /// Account ID
    pub account_id: ID,

    /// Parent ID - this is an ID external to the system
    /// It allows to link groups of events together to an outside entity
    pub parent_id: Option<String>,

    /// External ID - this is an ID external to the system
    /// It allows to link a group of events to an outside entity
    pub external_id: Option<String>,
}

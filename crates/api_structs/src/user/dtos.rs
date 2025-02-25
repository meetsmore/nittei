use nittei_domain::{ID, User};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// User object
#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserDTO {
    /// UUID of the user
    pub id: ID,

    /// External id
    #[ts(optional)]
    pub external_id: Option<String>,

    /// Metadata (e.g. {"key": "value"})
    #[ts(optional)]
    pub metadata: Option<serde_json::Value>,
}

impl UserDTO {
    pub fn new(user: User) -> Self {
        Self {
            id: user.id,
            external_id: user.external_id,
            metadata: user.metadata,
        }
    }
}

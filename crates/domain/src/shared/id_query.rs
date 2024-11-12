use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::ID;

/// Query parameters for searching on an ID
#[derive(Deserialize, Serialize, TS, Debug, Validate, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "IdQuery")]
pub struct IDQuery {
    /// Optional ID (equality test)
    pub eq: Option<ID>,
    /// Optional bool (existence test)
    /// If "eq" is provided, this field is ignored
    pub exists: Option<bool>,
}

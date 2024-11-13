use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

/// Query parameters for searching on an ID
#[derive(Deserialize, Serialize, TS, Debug, Validate, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "IDQuery")]
pub struct IDQuery {
    /// Optional String (equality test)
    /// This is not a UUID, but a string as we allow any type of ID in this field
    #[ts(optional)]
    pub eq: Option<String>,
    /// Optional bool (existence test)
    /// If "eq" is provided, this field is ignored
    #[ts(optional)]
    pub exists: Option<bool>,
}

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::ID;

/// Query parameters for searching on an ID (or list of IDs)
#[derive(Deserialize, Serialize, TS, Debug, Validate, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "IDQuery")]
pub struct IdQuery {
    /// Optional ID (equality test)
    #[ts(optional)]
    pub eq: Option<ID>,

    /// Optional ID (inequality test)
    /// If "eq" is provided, this field is ignored
    #[ts(optional)]
    pub ne: Option<ID>,

    /// Optional bool (existence test)
    /// If "eq" is provided, this field is ignored
    #[ts(optional)]
    pub exists: Option<bool>,

    /// Optional list of IDs (equality test)
    /// If "eq" is provided, this field is ignored
    /// (use r# in the field name as "in" is a reserved keyword)
    #[ts(optional)]
    pub r#in: Option<Vec<ID>>,
}

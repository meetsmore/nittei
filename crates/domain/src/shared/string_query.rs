use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

/// Query parameters for searching on a string
#[derive(Deserialize, Serialize, TS, Debug, Validate, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "IDQuery")]
pub struct StringQuery {
    /// Optional String (equality test)
    #[ts(optional)]
    pub eq: Option<String>,

    /// Optional string (inequality test)
    /// If "eq" is provided, this field is ignored
    #[ts(optional)]
    pub ne: Option<String>,

    /// Optional bool (existence test)
    /// If "eq" is provided, this field is ignored
    #[ts(optional)]
    pub exists: Option<bool>,

    /// Optional list of strings (equality test)
    /// If "eq" is provided, this field is ignored
    /// (use r# in the field name as "in" is a reserved keyword)
    #[ts(optional)]
    pub r#in: Option<Vec<String>>,
}

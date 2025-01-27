use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Query parameters for searching on a string
#[derive(Deserialize, Serialize, TS, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "StringQuery")]
pub enum StringQuery {
    /// Optional String (equality test)
    Eq(String),

    /// Optional string (inequality test)
    /// If "eq" is provided, this field is ignored
    Ne(String),

    /// Optional bool (existence test)
    /// If "eq" is provided, this field is ignored
    Exists(bool),

    /// Optional list of strings (equality test)
    /// If "eq" is provided, this field is ignored
    /// (use r# in the field name as "in" is a reserved keyword)
    In(Vec<String>),
}

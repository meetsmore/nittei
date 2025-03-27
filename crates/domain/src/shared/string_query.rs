use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

/// Query parameters for searching on a string
#[derive(Deserialize, Serialize, TS, Debug, Clone, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export, rename = "StringQuery", rename_all = "camelCase")]
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
    In(Vec<String>),
}

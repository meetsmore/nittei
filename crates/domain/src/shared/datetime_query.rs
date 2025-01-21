use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

/// Query parameters for searching on a date time
#[derive(Deserialize, Serialize, TS, Debug, Validate, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "DateTimeQuery")]
pub struct DateTimeQuery {
    /// Optional "equal" query (UTC)
    #[ts(type = "Date", optional)]
    pub eq: Option<DateTime<Utc>>,

    /// Optional "greater than or equal" query (UTC)
    #[ts(type = "Date", optional)]
    pub gte: Option<DateTime<Utc>>,

    /// Optional "less than or equal" query (UTC)
    #[ts(type = "Date", optional)]
    pub lte: Option<DateTime<Utc>>,

    /// Optional "greater than" query (UTC)
    /// This is exclusive of the value
    #[ts(type = "Date", optional)]
    pub gt: Option<DateTime<Utc>>,

    /// Optional "less than" query (UTC)
    /// This is exclusive of the value
    #[ts(type = "Date", optional)]
    pub lt: Option<DateTime<Utc>>,
}

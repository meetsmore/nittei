use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Deserialize, Serialize, TS, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "DateTimeQueryRange")]
pub struct DateTimeQueryRange {
    /// "greater than or equal" query (UTC)
    #[ts(type = "Date", optional)]
    pub gte: Option<DateTime<Utc>>,

    /// "less than or equal" query (UTC)
    #[ts(type = "Date", optional)]
    pub lte: Option<DateTime<Utc>>,

    /// "greater than" query (UTC)
    /// This is exclusive of the value
    #[ts(type = "Date", optional)]
    pub gt: Option<DateTime<Utc>>,

    /// "less than" query (UTC)
    /// This is exclusive of the value
    #[ts(type = "Date", optional)]
    pub lt: Option<DateTime<Utc>>,
}

/// Query parameters for searching on a date time
#[derive(Deserialize, Serialize, TS, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "DateTimeQuery")]
pub enum DateTimeQuery {
    /// "equal" query (UTC)
    #[ts(type = "Date")]
    Eq(DateTime<Utc>),

    Range(DateTimeQueryRange),
}

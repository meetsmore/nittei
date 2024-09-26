use std::error::Error;

use chrono::{prelude::*, DateTime};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A `TimeSpan` type represents a time interval (duration of time)
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimeSpan {
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    duration: i64,
}

impl TimeSpan {
    pub fn new(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        Self {
            start_time,
            end_time,
            duration: (end_time - start_time).num_milliseconds(),
        }
    }

    /// Duration of this `TimeSpan` is greater than a given duration
    pub fn greater_than(&self, duration: i64) -> bool {
        self.duration > duration
    }

    pub fn as_datetime(&self, tz: &Tz) -> TimeSpanDateTime {
        TimeSpanDateTime {
            start: self.start_time.with_timezone(tz),
            end: self.end_time.with_timezone(tz),
        }
    }

    pub fn start(&self) -> DateTime<Utc> {
        self.start_time
    }

    pub fn end(&self) -> DateTime<Utc> {
        self.end_time
    }
}

#[derive(Debug)]
pub struct InvalidTimeSpanError(i64, i64);

impl Error for InvalidTimeSpanError {}

impl std::fmt::Display for InvalidTimeSpanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Provided timespan start_ts: {} and end_ts: {} is invalid. It should be between 1 hour and 40 days.", self.0, self.1)
    }
}

#[derive(Debug)]
pub struct TimeSpanDateTime {
    pub start: DateTime<Tz>,
    pub end: DateTime<Tz>,
}

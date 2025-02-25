use chrono_tz::{Tz, UTC};
use serde::{Deserialize, Serialize};

use crate::{
    IntegrationProvider,
    Meta,
    Weekday,
    shared::entity::{Entity, ID},
};

#[derive(Debug, Clone, Default)]
pub struct Calendar {
    pub id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub name: Option<String>,
    pub key: Option<String>,
    pub settings: CalendarSettings,
    pub metadata: Option<serde_json::Value>,
}

impl Meta<ID> for Calendar {
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncedCalendar {
    pub provider: IntegrationProvider,
    pub calendar_id: ID,
    pub user_id: ID,
    pub ext_calendar_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSettings {
    pub week_start: Weekday,
    pub timezone: Tz,
}

impl Default for CalendarSettings {
    fn default() -> Self {
        Self {
            week_start: Weekday::Mon,
            timezone: UTC,
        }
    }
}

impl Calendar {
    pub fn new(user_id: &ID, account_id: &ID, name: Option<String>, key: Option<String>) -> Self {
        Self {
            id: Default::default(),
            user_id: user_id.clone(),
            account_id: account_id.clone(),
            name,
            key,
            settings: Default::default(),
            metadata: Default::default(),
        }
    }
}

impl Entity<ID> for Calendar {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

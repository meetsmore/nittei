use crate::{
    shared::{
        entity::{Entity, ID},
        metadata::Metadata,
    },
    Meta,
};
use chrono_tz::{Tz, UTC};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Calendar {
    pub id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub settings: CalendarSettings,
    pub synced: Vec<SyncedCalendar>,
    pub metadata: Metadata,
}

impl Meta<ID> for Calendar {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "provider", content = "id")]
pub enum SyncedCalendar {
    Google(String),
    Outlook(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarSettings {
    pub week_start: isize,
    pub timezone: Tz,
}

impl CalendarSettings {
    pub fn set_week_start(&mut self, wkst: isize) -> bool {
        if (0..=6).contains(&wkst) {
            self.week_start = wkst;
            true
        } else {
            false
        }
    }

    pub fn set_timezone(&mut self, timezone: &str) -> bool {
        match timezone.parse::<Tz>() {
            Ok(tzid) => {
                self.timezone = tzid;
                true
            }
            Err(_) => false,
        }
    }
}

impl Default for CalendarSettings {
    fn default() -> Self {
        Self {
            week_start: 0,
            timezone: UTC,
        }
    }
}

impl Calendar {
    pub fn new(user_id: &ID, account_id: &ID) -> Self {
        Self {
            id: Default::default(),
            user_id: user_id.clone(),
            account_id: account_id.clone(),
            settings: Default::default(),
            metadata: Default::default(),
            synced: Default::default(),
        }
    }

    pub fn get_outlook_calendar_ids(&self) -> Vec<String> {
        self.synced
            .iter()
            .filter_map(|cal| match cal {
                SyncedCalendar::Outlook(id) => Some(id.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    pub fn get_google_calendar_ids(&self) -> Vec<String> {
        self.synced
            .iter()
            .filter_map(|cal| match cal {
                SyncedCalendar::Google(id) => Some(id.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    }
}

impl Entity<ID> for Calendar {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

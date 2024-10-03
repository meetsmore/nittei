use std::convert::TryFrom;

use chrono::{prelude::*, Duration, TimeDelta};
use rrule::RRuleSet;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    calendar::CalendarSettings,
    event_instance::EventInstance,
    shared::{
        entity::{Entity, ID},
        metadata::Metadata,
        recurrence::RRuleOptions,
    },
    timespan::TimeSpan,
    IntegrationProvider,
    Meta,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CalendarEventStatus {
    #[default]
    Tentative,
    Confirmed,
    Cancelled,
}

impl From<CalendarEventStatus> for String {
    fn from(e: CalendarEventStatus) -> Self {
        match e {
            CalendarEventStatus::Tentative => "tentative".into(),
            CalendarEventStatus::Confirmed => "confirmed".into(),
            CalendarEventStatus::Cancelled => "cancelled".into(),
        }
    }
}

impl TryFrom<String> for CalendarEventStatus {
    type Error = anyhow::Error;
    fn try_from(e: String) -> anyhow::Result<CalendarEventStatus> {
        Ok(match &e[..] {
            "tentative" => CalendarEventStatus::Tentative,
            "confirmed" => CalendarEventStatus::Confirmed,
            "cancelled" => CalendarEventStatus::Cancelled,
            _ => Err(anyhow::anyhow!("Invalid status"))?,
        })
    }
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: ID,
    pub parent_id: Option<String>,
    pub external_id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub all_day: bool,
    pub status: CalendarEventStatus,
    pub start_time: DateTime<Utc>,
    pub duration: i64,
    pub busy: bool,
    pub end_time: DateTime<Utc>,
    pub created: i64,
    pub updated: i64,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<DateTime<Utc>>,
    pub calendar_id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub reminders: Vec<CalendarEventReminder>,
    pub service_id: Option<ID>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncedCalendarEvent {
    pub event_id: ID,
    pub calendar_id: ID,
    pub user_id: ID,
    pub ext_event_id: String,
    pub ext_calendar_id: String,
    pub provider: IntegrationProvider,
}

impl Entity<ID> for CalendarEvent {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

impl Meta<ID> for CalendarEvent {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }
    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CalendarEventReminder {
    pub delta: i64, // In minutes
    pub identifier: String,
}

impl CalendarEventReminder {
    // This isn't ideal at all, shouldn't be possible to construct
    // this type of it is not valid, but for now it is good enough
    pub fn is_valid(&self) -> bool {
        let max_delta = 60 * 24 * 31 * 12; // More than one year in minutes
        self.delta >= -max_delta && self.delta <= max_delta
    }
}

impl CalendarEvent {
    fn update_endtime(&mut self, calendar_settings: &CalendarSettings) -> bool {
        match self.recurrence.clone() {
            Some(recurrence) => {
                let rrule_options =
                    recurrence.get_parsed_options(self.start_time, calendar_settings);
                let options = rrule_options.get_rrule().first();
                if let Some(options) = options {
                    if (options.get_count().unwrap_or(0) > 0) || options.get_until().is_some() {
                        let expand = self.expand(None, calendar_settings);
                        self.end_time = expand
                            .last()
                            .map(|l| l.end_time)
                            .unwrap_or(DateTime::<Utc>::MIN_UTC);
                    } else {
                        self.end_time = DateTime::<Utc>::MAX_UTC;
                    }
                } else {
                    self.end_time = DateTime::<Utc>::MAX_UTC;
                }
                true
            }
            None => true,
        }
    }

    pub fn set_recurrence(
        &mut self,
        recurrence: RRuleOptions,
        calendar_settings: &CalendarSettings,
        update_endtime: bool,
    ) -> bool {
        let valid_recurrence = recurrence.is_valid(self.start_time);
        if !valid_recurrence {
            return false;
        }

        self.recurrence = Some(recurrence);
        if update_endtime {
            return self.update_endtime(calendar_settings);
        }
        true
    }

    pub fn get_rrule_set(&self, calendar_settings: &CalendarSettings) -> Option<RRuleSet> {
        if let Some(recurrence) = self.recurrence.clone() {
            let rrule_options = recurrence.get_parsed_options(self.start_time, calendar_settings);
            let tzid = rrule_options.get_dt_start().timezone();
            let mut rrule_set = RRuleSet::new(*rrule_options.get_dt_start());
            for exdate in &self.exdates {
                rrule_set = rrule_set.exdate(exdate.with_timezone(&tzid));
            }
            let rrule = rrule_options.get_rrule().first()?;
            rrule_set = rrule_set.rrule(rrule.clone());
            Some(rrule_set)
        } else {
            None
        }
    }

    pub fn expand(
        &self,
        timespan: Option<&TimeSpan>,
        calendar_settings: &CalendarSettings,
    ) -> Vec<EventInstance> {
        match &self.recurrence {
            Some(recurrence) => {
                let rrule_options =
                    recurrence.get_parsed_options(self.start_time, calendar_settings);
                let tzid = rrule_options.get_dt_start().timezone();
                let rrule_set = match self.get_rrule_set(calendar_settings) {
                    Some(rrule_set) => rrule_set,
                    None => return Vec::new(),
                };

                let instances = match timespan {
                    Some(timespan) => {
                        let chrono_tz = match tzid {
                            rrule::Tz::Tz(tz) => tz,
                            rrule::Tz::Local(_) => chrono_tz::UTC,
                        };
                        let timespan = timespan.as_datetime(&chrono_tz);

                        // Also take the duration of events into consideration as the rrule library
                        // does not support duration on events.
                        let end = timespan.end - Duration::milliseconds(self.duration);

                        // RRule v0.5.5 is not inclusive on start, so just by subtracting one millisecond
                        // will make it inclusive
                        let start = timespan.start - Duration::milliseconds(1);

                        rrule_set
                            .before(end.with_timezone(&tzid))
                            .after(start.with_timezone(&tzid))
                            .all(100)
                    }
                    None => rrule_set.all(100), // TODO: change
                };

                instances
                    .dates
                    .iter()
                    .map(|occurrence| EventInstance {
                        start_time: occurrence.with_timezone(&Utc),
                        end_time: occurrence.with_timezone(&Utc)
                            + TimeDelta::milliseconds(self.duration),
                        busy: self.busy,
                    })
                    .collect()
            }
            None => {
                if self.exdates.contains(&self.start_time) {
                    Vec::new()
                } else {
                    vec![EventInstance {
                        start_time: self.start_time,
                        end_time: self.start_time + TimeDelta::milliseconds(self.duration),
                        busy: self.busy,
                    }]
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use chrono_tz::UTC;

    use super::*;
    use crate::{shared::recurrence::WeekDayRecurrence, RRuleFrequency};

    #[test]
    fn daily_calendar_event() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: Weekday::Mon,
        };
        let event = CalendarEvent {
            start_time: DateTime::from_timestamp_millis(1521317491239).unwrap(),
            duration: 1000 * 60 * 60,
            recurrence: Some(RRuleOptions {
                freq: RRuleFrequency::Daily,
                interval: 1,
                count: Some(4),
                ..Default::default()
            }),
            end_time: DateTime::from_timestamp_millis(2521317491239).unwrap(),
            exdates: vec![DateTime::from_timestamp_millis(1521317491239).unwrap()],
            ..Default::default()
        };

        let oc = event.expand(None, &settings);

        // To double check, it was 3 before
        // Imo it makes sense to have 4, as count is 4 and the end_time is very far away
        // The argument is maybe that the exdate == start_time, but if now, it removes it from the count
        // Then it's due to a change in RRULE library
        assert_eq!(oc.len(), 4);
    }

    #[test]
    fn calendar_event_without_recurrence() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: Weekday::Mon,
        };
        let mut event = CalendarEvent {
            start_time: DateTime::from_timestamp_millis(1521317491239).unwrap(),
            duration: 1000 * 60 * 60,
            end_time: DateTime::from_timestamp_millis(2521317491239).unwrap(),
            ..Default::default()
        };

        let oc = event.expand(None, &settings);
        assert_eq!(oc.len(), 1);

        // Without recurrence but with exdate at start time
        event.exdates = vec![event.start_time];
        let oc = event.expand(None, &settings);
        assert_eq!(oc.len(), 0);
    }

    #[test]
    fn rejects_event_with_invalid_recurrence() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: Weekday::Mon,
        };
        let mut invalid_rrules = Vec::new();
        invalid_rrules.push(RRuleOptions {
            count: Some(1000), // too big count
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            until: Some(Utc.with_ymd_and_hms(2150, 1, 1, 0, 0, 0).unwrap()), // too big until
            ..Default::default()
        });
        invalid_rrules.push(RRuleOptions {
            // Only bysetpos and no by*
            bysetpos: Some(vec![1]),
            freq: RRuleFrequency::Monthly,
            ..Default::default()
        });
        for rrule in invalid_rrules {
            let mut event = CalendarEvent {
                start_time: DateTime::from_timestamp_millis(1521317491239).unwrap(),
                duration: 1000 * 60 * 60,
                end_time: DateTime::from_timestamp_millis(2521317491239).unwrap(),
                ..Default::default()
            };

            assert!(!event.set_recurrence(rrule, &settings, true));
        }
    }

    #[test]
    fn allows_event_with_valid_recurrence() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: Weekday::Mon,
        };
        let mut valid_rrules = Vec::new();
        let start_time = DateTime::from_timestamp_millis(1521317491239).unwrap();
        valid_rrules.push(Default::default());
        valid_rrules.push(RRuleOptions {
            count: Some(100),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            until: Some(start_time + TimeDelta::milliseconds(1000 * 60 * 60 * 24 * 100)),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            byweekday: Some(vec![WeekDayRecurrence::new(Weekday::Tue).unwrap()]),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            byweekday: Some(vec![WeekDayRecurrence::new_nth(Weekday::Tue, 1).unwrap()]),
            freq: RRuleFrequency::Monthly,
            ..Default::default()
        });
        for rrule in valid_rrules {
            let mut event = CalendarEvent {
                start_time,
                duration: 1000 * 60 * 60,
                end_time: DateTime::from_timestamp_millis(2521317491239).unwrap(),
                ..Default::default()
            };

            assert!(event.set_recurrence(rrule, &settings, true));
        }
    }
}

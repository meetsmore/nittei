use std::convert::TryFrom;

use chrono::{prelude::*, TimeDelta};
use rrule::RRuleSet;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    calendar::CalendarSettings,
    event_instance::EventInstance,
    shared::{
        entity::{Entity, ID},
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
    pub event_type: Option<String>,
    pub location: Option<String>,
    pub all_day: bool,
    pub status: CalendarEventStatus,
    pub start_time: DateTime<Utc>,
    pub duration: i64,
    pub busy: bool,
    pub end_time: DateTime<Utc>,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<DateTime<Utc>>,
    pub recurring_event_id: Option<ID>,
    pub original_start_time: Option<DateTime<Utc>>,
    pub calendar_id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub reminders: Vec<CalendarEventReminder>,
    pub service_id: Option<ID>,
    pub group_id: Option<ID>,
    pub metadata: Option<serde_json::Value>,
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
    fn update_endtime(&mut self, calendar_settings: &CalendarSettings) -> anyhow::Result<bool> {
        match self.recurrence.clone() {
            Some(recurrence) => {
                let rrule_options =
                    recurrence.get_parsed_options(self.start_time, calendar_settings)?;
                let options = rrule_options.get_rrule().first();
                if let Some(options) = options {
                    if (options.get_count().unwrap_or(0) > 0) || options.get_until().is_some() {
                        let expand = self.expand(None, calendar_settings)?;
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
                Ok(true)
            }
            None => Ok(true),
        }
    }

    pub fn set_recurrence(
        &mut self,
        recurrence: RRuleOptions,
        calendar_settings: &CalendarSettings,
        update_endtime: bool,
    ) -> anyhow::Result<bool> {
        let valid_recurrence = recurrence.is_valid(self.start_time);
        if !valid_recurrence {
            return Ok(false);
        }

        self.recurrence = Some(recurrence);
        if update_endtime {
            return self.update_endtime(calendar_settings);
        }
        Ok(true)
    }

    pub fn get_rrule_set(
        &self,
        calendar_settings: &CalendarSettings,
    ) -> anyhow::Result<Option<RRuleSet>> {
        if let Some(recurrence) = self.recurrence.clone() {
            let rrule_options =
                recurrence.get_parsed_options(self.start_time, calendar_settings)?;
            let tzid = rrule_options.get_dt_start().timezone();
            let mut rrule_set = RRuleSet::new(*rrule_options.get_dt_start());
            for exdate in &self.exdates {
                rrule_set = rrule_set.exdate(exdate.with_timezone(&tzid));
            }
            let rrule = rrule_options.get_rrule().first();
            match rrule {
                Some(rrule) => {
                    rrule_set = rrule_set.rrule(rrule.clone());
                    Ok(Some(rrule_set))
                }
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub fn expand(
        &self,
        timespan: Option<&TimeSpan>,
        calendar_settings: &CalendarSettings,
    ) -> anyhow::Result<Vec<EventInstance>> {
        match &self.recurrence {
            Some(recurrence) => {
                let rrule_options =
                    recurrence.get_parsed_options(self.start_time, calendar_settings)?;
                let tzid = rrule_options.get_dt_start().timezone();
                let rrule_set = match self.get_rrule_set(calendar_settings)? {
                    Some(rrule_set) => rrule_set,
                    None => return Ok(Vec::new()),
                };

                let instances = match timespan {
                    Some(timespan) => {
                        let chrono_tz = match tzid {
                            rrule::Tz::Tz(tz) => tz,
                            rrule::Tz::Local(_) => chrono_tz::UTC,
                        };

                        let timespan = timespan.as_datetime(&chrono_tz);

                        let end_with_timezone = timespan.end.with_timezone(&tzid);
                        let start_with_timezone = timespan.start.with_timezone(&tzid);

                        rrule_set
                            .after(start_with_timezone)
                            .before(end_with_timezone)
                            .all(100)
                    }
                    None => rrule_set.all(100), // TODO: change
                };

                Ok(instances
                    .dates
                    .iter()
                    .map(|occurrence| EventInstance {
                        start_time: occurrence.with_timezone(&Utc),
                        end_time: occurrence.with_timezone(&Utc)
                            + TimeDelta::milliseconds(self.duration),
                        busy: self.busy,
                    })
                    .collect())
            }
            None => {
                if self.exdates.contains(&self.start_time) {
                    Ok(Vec::new())
                } else {
                    Ok(vec![EventInstance {
                        start_time: self.start_time,
                        end_time: self.start_time + TimeDelta::milliseconds(self.duration),
                        busy: self.busy,
                    }])
                }
            }
        }
    }

    /// Filters the instances based on the changed instances, and returns the instances that are not changed
    /// The changed instances are actual event calendars (contrary to normal instances)
    /// They need to be removed from the default instances list, as they are not part of the recurrence anymore
    pub fn remove_changed_instances(
        &self,
        instances: Vec<EventInstance>,
        exceptions_start_times: &[DateTime<Utc>],
    ) -> Vec<EventInstance> {
        let original_start_times_set: std::collections::HashSet<DateTime<Utc>> =
            exceptions_start_times.iter().cloned().collect();

        instances
            .iter()
            .filter(|instance| !original_start_times_set.contains(&instance.start_time))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod test {
    use core::panic;

    use chrono::Duration;
    use chrono_tz::UTC;

    use super::*;
    use crate::{shared::recurrence::WeekDayRecurrence, RRuleFrequency};

    #[test]
    fn daily_calendar_event() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: Weekday::Mon,
        };
        let start_time = DateTime::from_timestamp_millis(1521317491000).unwrap();
        let event = CalendarEvent {
            start_time,
            duration: 1000 * 60 * 60,
            recurrence: Some(RRuleOptions {
                freq: RRuleFrequency::Daily,
                interval: 1,
                count: Some(4),
                ..Default::default()
            }),
            end_time: DateTime::from_timestamp_millis(2521317491239).unwrap(),
            exdates: vec![start_time],
            ..Default::default()
        };

        let oc = match event.expand(None, &settings) {
            Ok(oc) => oc,
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };

        // We expect 3 occurrences, as the count is 4 and 1 is removed due to exdate
        assert_eq!(oc.len(), 3);
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

        let oc = match event.expand(None, &settings) {
            Ok(oc) => oc,
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };
        assert_eq!(oc.len(), 1);

        // Without recurrence but with exdate at start time
        event.exdates = vec![event.start_time];
        let oc = match event.expand(None, &settings) {
            Ok(oc) => oc,
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };
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

            let valid = match event.set_recurrence(rrule, &settings, true) {
                Ok(valid) => valid,
                Err(e) => {
                    panic!("Error: {:?}", e);
                }
            };
            assert!(!valid);
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

            let valid = match event.set_recurrence(rrule, &settings, true) {
                Ok(valid) => valid,
                Err(e) => {
                    panic!("Error: {:?}", e);
                }
            };
            assert!(valid);
        }
    }

    #[test]
    fn daily_calendar_event_with_some_overrides() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: Weekday::Mon,
        };
        let start_time = DateTime::parse_from_rfc3339("2025-02-02T10:00:00+00:00")
            .unwrap()
            .to_utc();

        let event = CalendarEvent {
            start_time,
            duration: 1000 * 60 * 60,
            recurrence: Some(RRuleOptions {
                freq: RRuleFrequency::Daily,
                interval: 1,
                count: Some(4),
                ..Default::default()
            }),
            end_time: DateTime::parse_from_rfc3339("2026-02-02T10:00:00+00:00")
                .unwrap()
                .to_utc(),
            exdates: vec![start_time],
            ..Default::default()
        };

        let oc = match event.expand(None, &settings) {
            Ok(oc) => oc,
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };

        // We expect 3 occurrences, as the count is 4, and 1 is removed due to exdate
        assert_eq!(oc.len(), 3);

        let event_override_changed = CalendarEvent {
            start_time: start_time + Duration::days(1) + Duration::hours(1),
            duration: 1000 * 60 * 60,
            end_time: start_time + Duration::days(1) + Duration::hours(2),
            original_start_time: Some(start_time + Duration::days(1)),
            recurring_event_id: Some(event.id.clone()),
            ..Default::default()
        };

        let event_override_cancelled = CalendarEvent {
            start_time: start_time + Duration::days(2),
            duration: 1000 * 60 * 60,
            end_time: start_time + Duration::days(2) + Duration::hours(1),
            original_start_time: Some(start_time + Duration::days(2)),
            recurring_event_id: Some(event.id.clone()),
            status: CalendarEventStatus::Cancelled,
            ..Default::default()
        };

        let oc_filtered = event.remove_changed_instances(
            oc,
            &[
                event_override_changed.original_start_time.unwrap(),
                event_override_cancelled.original_start_time.unwrap(),
            ],
        );

        // We expect 1 occurrences, as the count is 4, we have 1 exdate and 2 overrides
        // The overrides are not included in the occurrences, whatever the change is
        // As they, by themselves, already "represent" these instances
        assert_eq!(oc_filtered.len(), 1);
    }
}

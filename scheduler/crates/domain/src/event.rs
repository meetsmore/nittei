use chrono::{prelude::*, Duration};
use rrule::{RRule, RRuleSet};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default)]
pub struct CalendarEvent {
    pub id: ID,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub end_ts: i64,
    pub created: i64,
    pub updated: i64,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<i64>,
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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
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
                let rrule_options = recurrence.get_parsed_options(self.start_ts, calendar_settings);
                if (rrule_options.count.is_some() && rrule_options.count.unwrap() > 0)
                    || rrule_options.until.is_some()
                {
                    let expand = self.expand(None, calendar_settings);
                    self.end_ts = expand.last().map(|l| l.end_ts).unwrap_or(0);
                } else {
                    self.end_ts = Self::get_max_timestamp();
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
        let valid_recurrence = recurrence.is_valid(self.start_ts);
        if !valid_recurrence {
            return false;
        }

        self.recurrence = Some(recurrence);
        if update_endtime {
            return self.update_endtime(calendar_settings);
        }
        true
    }

    pub fn get_max_timestamp() -> i64 {
        5609882500905 // Mon Oct 09 2147 06:41:40 GMT+0200 (Central European Summer Time)
    }

    pub fn get_rrule_set(&self, calendar_settings: &CalendarSettings) -> Option<RRuleSet> {
        self.recurrence.clone().map(|recurrence| {
            let rrule_options = recurrence.get_parsed_options(self.start_ts, calendar_settings);
            let tzid = rrule_options.tzid;
            let mut rrule_set = RRuleSet::new();
            for exdate in &self.exdates {
                let exdate = tzid.timestamp_millis(*exdate);
                rrule_set.exdate(exdate);
            }
            let rrule = RRule::new(rrule_options);
            rrule_set.rrule(rrule);
            rrule_set
        })
    }

    pub fn expand(
        &self,
        timespan: Option<&TimeSpan>,
        calendar_settings: &CalendarSettings,
    ) -> Vec<EventInstance> {
        match self.recurrence.clone() {
            Some(recurrence) => {
                let rrule_options = recurrence.get_parsed_options(self.start_ts, calendar_settings);
                let tzid = rrule_options.tzid;
                let rrule_set = self.get_rrule_set(calendar_settings).unwrap();

                let instances = match timespan {
                    Some(timespan) => {
                        let timespan = timespan.as_datetime(&tzid);

                        // Also take the duration of events into consideration as the rrule library
                        // does not support duration on events.
                        let end = timespan.end - Duration::milliseconds(self.duration);

                        // RRule v0.5.5 is not inclusive on start, so just by subtracting one millisecond
                        // will make it inclusive
                        let start = timespan.start - Duration::milliseconds(1);

                        rrule_set.between(start, end, true)
                    }
                    None => rrule_set.all(),
                };

                instances
                    .iter()
                    .map(|occurrence| {
                        let start_ts = occurrence.timestamp_millis();

                        EventInstance {
                            start_ts,
                            end_ts: start_ts + self.duration,
                            busy: self.busy,
                        }
                    })
                    .collect()
            }
            None => {
                if self.exdates.contains(&self.start_ts) {
                    Vec::new()
                } else {
                    vec![EventInstance {
                        start_ts: self.start_ts,
                        end_ts: self.start_ts + self.duration,
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
    use crate::{shared::recurrence::WeekDay, RRuleFrequency};

    #[test]
    fn daily_calendar_event() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: Weekday::Mon,
        };
        let event = CalendarEvent {
            start_ts: 1521317491239,
            duration: 1000 * 60 * 60,
            recurrence: Some(RRuleOptions {
                freq: RRuleFrequency::Daily,
                interval: 1,
                count: Some(4),
                ..Default::default()
            }),
            end_ts: 2521317491239,
            exdates: vec![1521317491239],
            ..Default::default()
        };

        let oc = event.expand(None, &settings);
        assert_eq!(oc.len(), 3);
    }

    #[test]
    fn calendar_event_without_recurrence() {
        let settings = CalendarSettings {
            timezone: UTC,
            week_start: Weekday::Mon,
        };
        let mut event = CalendarEvent {
            start_ts: 1521317491239,
            duration: 1000 * 60 * 60,
            end_ts: 2521317491239,
            ..Default::default()
        };

        let oc = event.expand(None, &settings);
        assert_eq!(oc.len(), 1);

        // Without recurrence but with exdate at start time
        event.exdates = vec![event.start_ts];
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
            until: Some(Utc.ymd(2150, 1, 1).and_hms(0, 0, 0).timestamp_millis() as isize), // too big until
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
                start_ts: 1521317491239,
                duration: 1000 * 60 * 60,
                end_ts: 2521317491239,
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
        let start_ts = 1521317491239;
        valid_rrules.push(Default::default());
        valid_rrules.push(RRuleOptions {
            count: Some(100),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            until: Some(start_ts + 1000 * 60 * 60 * 24 * 100),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            byweekday: Some(vec![WeekDay::new(Weekday::Tue)]),
            ..Default::default()
        });
        valid_rrules.push(RRuleOptions {
            byweekday: Some(vec![WeekDay::new_nth(Weekday::Tue, 1).unwrap()]),
            freq: RRuleFrequency::Monthly,
            ..Default::default()
        });
        for rrule in valid_rrules {
            let mut event = CalendarEvent {
                start_ts: start_ts as i64,
                duration: 1000 * 60 * 60,
                end_ts: 2521317491239,
                ..Default::default()
            };

            assert!(event.set_recurrence(rrule, &settings, true));
        }
    }
}

use std::{collections::HashMap, str::FromStr};

use chrono::{offset::LocalResult, prelude::*, Duration};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

use crate::{
    date,
    event_instance::EventInstance,
    shared::entity::{Entity, ID},
    timespan::TimeSpan,
    CompatibleInstances,
    Meta,
    Metadata,
};

#[derive(Debug, Clone)]
pub struct Schedule {
    pub id: ID,
    pub user_id: ID,
    pub account_id: ID,
    pub rules: Vec<ScheduleRule>,
    pub timezone: Tz,
    pub metadata: Metadata,
}

impl Meta<ID> for Schedule {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn account_id(&self) -> &ID {
        &self.account_id
    }
}

impl Schedule {
    pub fn new(user_id: ID, account_id: ID, timezone: &Tz) -> Self {
        Self {
            id: Default::default(),
            user_id,
            account_id,
            rules: ScheduleRule::default_rules(),
            timezone: timezone.to_owned(),
            metadata: Default::default(),
        }
    }

    pub fn set_rules(&mut self, rules: &[ScheduleRule]) {
        let now = Utc::now();
        let min_date = now - Duration::days(2);

        #[allow(clippy::unwrap_used)]
        let max_date = (now + Duration::days(365 * 5))
            .with_month(1)
            .unwrap()
            .with_day(1)
            .unwrap();
        let allowed_rules = rules
            .iter()
            .filter(|&r| match &r.variant {
                ScheduleRuleVariant::Date(datestr) => match datestr.parse::<Day>() {
                    Ok(day) => {
                        let date = day.date(&self.timezone);
                        date > min_date && date < max_date
                    }
                    Err(_) => false,
                },
                _ => true,
            })
            .cloned()
            .map(|mut r| {
                r.parse_intervals();
                r
            })
            .collect();
        self.rules = allowed_rules;
    }
}

impl Entity<ID> for Schedule {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "value")]
pub enum ScheduleRuleVariant {
    WDay(Weekday),
    Date(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
struct Time {
    pub hours: i64,
    pub minutes: i64,
}

impl std::cmp::PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.hours.cmp(&other.hours) {
            std::cmp::Ordering::Less => return Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Greater => return Some(std::cmp::Ordering::Greater),
            _ => (),
        };

        Some(self.minutes.cmp(&other.minutes))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ScheduleRuleInterval {
    start: Time,
    end: Time,
}

impl ScheduleRuleInterval {
    /// Creates an `EventInstance` if the given timerange exists within
    /// that `Day` in the given timezone.
    /// If it is possible to create a timerange that is smaller but
    /// but still within the origin timerange then that timerange will be
    /// returned.
    pub fn to_event(&self, day: &Day, tzid: &Tz) -> Option<EventInstance> {
        let mut start_hours = self.start.hours as u32;
        let date = tzid
            .with_ymd_and_hms(day.year, day.month, day.day, 0, 0, 0)
            .unwrap();

        // Try to find an hour in this day that is not invalid (DST) and greater than
        // or equals to `self.start.hour`

        let mut start_naive_time =
            NaiveTime::from_hms_opt(start_hours, self.start.minutes as u32, 0);

        let mut start = if let Some(start_naive_time) = start_naive_time {
            date.with_time(start_naive_time)
        } else {
            LocalResult::None
        };
        while start_naive_time.is_none() || start.single().is_none() {
            start_hours = (start_hours + 1) % 24;
            // Minutes can be zero now
            start_naive_time = NaiveTime::from_hms_opt(start_hours, self.start.minutes as u32, 0);
            start = if let Some(start_naive_time) = start_naive_time {
                date.with_time(start_naive_time)
            } else {
                LocalResult::None
            };
        }
        let start = start.unwrap();
        if self.start.hours as u32 > start.hour() {
            // No valid start for this date
            return None;
        }

        // Try to find an hour in this day that is not invalid (DST) and less than
        // or equals to `self.end.hour`
        let end_hours = self.end.hours as u32;

        let mut end_naive_time = NaiveTime::from_hms_opt(end_hours, self.end.minutes as u32, 0);
        let mut end = if let Some(end_naive_time) = end_naive_time {
            date.with_time(end_naive_time)
        } else {
            LocalResult::None
        };
        while end_naive_time.is_none() || end.single().is_none() {
            start_hours = if start_hours == 0 {
                23
            } else {
                start_hours - 1
            };

            end_naive_time = NaiveTime::from_hms_opt(end_hours, self.end.minutes as u32, 0);
            end = if let Some(end_naive_time) = end_naive_time {
                date.with_time(end_naive_time)
            } else {
                LocalResult::None
            };
        }
        let end = end.unwrap();
        if end.hour() < self.end.hours as u32 {
            // No valid end hours for this date
            return None;
        }

        // Start should not be greater than end
        if start > end {
            return None;
        }

        Some(EventInstance {
            busy: false,
            start_time: start.with_timezone(&chrono::Utc),
            end_time: end.with_timezone(&chrono::Utc),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScheduleRule {
    pub variant: ScheduleRuleVariant,
    pub intervals: Vec<ScheduleRuleInterval>,
}

impl ScheduleRule {
    fn default_rules() -> Vec<Self> {
        let mut weekly_rules = Vec::new();
        let weekdays = vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ];
        for wday in weekdays {
            let intervals = match wday {
                Weekday::Sat | Weekday::Sun => Vec::new(),
                _ => vec![ScheduleRuleInterval {
                    start: Time {
                        hours: 9,
                        minutes: 0,
                    },
                    end: Time {
                        hours: 17,
                        minutes: 30,
                    },
                }],
            };
            weekly_rules.push(ScheduleRule {
                variant: ScheduleRuleVariant::WDay(wday),
                intervals,
            });
        }
        weekly_rules
    }

    fn parse_intervals(&mut self) {
        if self.intervals.len() > 10 {
            self.intervals.splice(10.., Vec::new());
        }
        // earliest start first
        // TODO: to fix
        #[allow(clippy::unwrap_used)]
        self.intervals
            .sort_by(|i1, i2| i1.start.partial_cmp(&i2.start).unwrap());

        self.intervals
            .retain(|interval| interval.start <= interval.end);

        let mut remove_intervals = HashMap::new();

        for i in 0..self.intervals.len() {
            if remove_intervals.contains_key(&i) {
                continue;
            }
            for j in (i + 1)..self.intervals.len() {
                if remove_intervals.contains_key(&j) {
                    continue;
                }
                if self.intervals[j].start == self.intervals[i].start
                    || self.intervals[j].start <= self.intervals[i].end
                {
                    if self.intervals[j].end > self.intervals[i].end {
                        self.intervals[i].end = self.intervals[j].end.clone();
                    }
                    remove_intervals.insert(j, true);
                }
            }
        }

        let mut remove_intervals = remove_intervals.keys().copied().collect::<Vec<_>>();
        // largest index first
        // TODO: to fix
        #[allow(clippy::unwrap_used)]
        remove_intervals.sort_by(|i1, i2| i2.partial_cmp(i1).unwrap());
        for index in remove_intervals {
            self.intervals.remove(index);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Day {
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl FromStr for Day {
    type Err = ();

    fn from_str(datestr: &str) -> Result<Self, Self::Err> {
        date::is_valid_date(datestr)
            .map(|(year, month, day)| Day { year, month, day })
            .map_err(|_| ())
    }
}

impl Day {
    pub fn inc(&mut self) {
        if self.day == date::get_month_length(self.year, self.month) {
            self.day = 1;
            if self.month == 12 {
                self.month = 1;
                self.year += 1;
            } else {
                self.month += 1;
            }
        } else {
            self.day += 1;
        }
    }

    pub fn weekday(&self, tzid: &Tz) -> Weekday {
        self.date(tzid).weekday()
    }

    pub fn date(&self, tzid: &Tz) -> DateTime<Tz> {
        tzid.with_ymd_and_hms(self.year, self.month, self.day, 0, 0, 0)
            .unwrap()
    }
}

impl std::fmt::Display for Day {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}-{}", self.year, self.month, self.day)
    }
}

impl std::cmp::PartialOrd for Day {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.year.cmp(&other.year) {
            std::cmp::Ordering::Less => return Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Greater => return Some(std::cmp::Ordering::Greater),
            _ => (),
        };
        match self.month.cmp(&other.month) {
            std::cmp::Ordering::Less => return Some(std::cmp::Ordering::Less),
            std::cmp::Ordering::Greater => return Some(std::cmp::Ordering::Greater),
            _ => (),
        };
        Some(self.day.cmp(&other.day))
    }
}

impl Schedule {
    pub fn freebusy(&self, timespan: &TimeSpan) -> CompatibleInstances {
        let start = timespan.start().with_timezone(&self.timezone);
        let end = timespan.end().with_timezone(&self.timezone);

        let mut date_lookup = HashMap::new();
        let mut weekday_lookup = HashMap::new();
        for rule in &self.rules {
            match &rule.variant {
                ScheduleRuleVariant::Date(date) => {
                    date_lookup.insert(date, &rule.intervals);
                }
                ScheduleRuleVariant::WDay(wkay) => {
                    weekday_lookup.insert(wkay, &rule.intervals);
                }
            }
        }

        let mut free_instances = CompatibleInstances::new(Vec::new());

        let mut day_cursor = Day {
            year: start.year(),
            month: start.month(),
            day: start.day(),
        };
        let last_day = Day {
            year: end.year(),
            month: end.month(),
            day: end.day(),
        };

        while day_cursor <= last_day {
            let day_str = day_cursor.to_string();

            let intervals = match date_lookup.get(&day_str) {
                Some(intervals) => Some(intervals),
                None => {
                    // check if weekday rule exists
                    let weekday = day_cursor.weekday(&self.timezone);
                    weekday_lookup.get(&weekday)
                }
            };
            if let Some(intervals) = intervals {
                for interval in intervals.iter() {
                    if let Some(event) = interval.to_event(&day_cursor, &self.timezone) {
                        free_instances.push_back(event);
                    }
                }
            }
            day_cursor.inc();
        }
        std::mem::drop(date_lookup);

        // Make sure all generated instances are within the timespan
        free_instances.remove_all_before(timespan.start());
        free_instances.remove_all_after(timespan.end());

        free_instances
    }
}

#[cfg(test)]
mod test {
    use chrono::TimeDelta;

    use super::*;

    #[test]
    fn day_sanity_tests() {
        let mut day = Day {
            year: 2021,
            month: 1,
            day: 1,
        };
        day.inc();
        assert_eq!(
            day,
            Day {
                year: 2021,
                month: 1,
                day: 2
            }
        );
        let mut day = Day {
            year: 2021,
            month: 1,
            day: 31,
        };
        day.inc();
        assert_eq!(
            day,
            Day {
                year: 2021,
                month: 2,
                day: 1
            }
        );
        let mut day = Day {
            year: 2021,
            month: 12,
            day: 31,
        };
        day.inc();
        assert_eq!(
            day,
            Day {
                year: 2022,
                month: 1,
                day: 1
            }
        );
        for _ in 0..365 {
            day.inc();
        }
        assert_eq!(
            day,
            Day {
                year: 2023,
                month: 1,
                day: 1
            }
        );
    }

    #[test]
    fn it_computes_freebusy_for_schedule() {
        let schedule = Schedule {
            id: Default::default(),
            user_id: Default::default(),
            account_id: Default::default(),
            timezone: chrono_tz::UTC,
            rules: vec![
                ScheduleRule {
                    variant: ScheduleRuleVariant::WDay(Weekday::Mon),
                    intervals: vec![ScheduleRuleInterval {
                        start: Time {
                            hours: 8,
                            minutes: 0,
                        },
                        end: Time {
                            hours: 10,
                            minutes: 30,
                        },
                    }],
                },
                ScheduleRule {
                    variant: ScheduleRuleVariant::Date("1970-1-12".into()),
                    intervals: vec![ScheduleRuleInterval {
                        start: Time {
                            hours: 9,
                            minutes: 0,
                        },
                        end: Time {
                            hours: 12,
                            minutes: 30,
                        },
                    }],
                },
            ],
            metadata: Default::default(),
        };

        let timespan = TimeSpan::new(
            DateTime::from_timestamp_millis(0).unwrap(),
            DateTime::from_timestamp_millis(1000 * 60 * 60 * 24 * 30).unwrap(),
        );
        let freebusy = schedule.freebusy(&timespan).inner();

        assert_eq!(freebusy.len(), 4);
        assert_eq!(
            freebusy[0],
            EventInstance {
                start_time: DateTime::from_timestamp_millis(374400000).unwrap(),
                end_time: DateTime::from_timestamp_millis(383400000).unwrap(),
                busy: false
            }
        );
        // Check that Date variant overrides wday variant
        assert_eq!(
            freebusy[1],
            EventInstance {
                start_time: DateTime::from_timestamp_millis(982800000).unwrap(),
                end_time: DateTime::from_timestamp_millis(995400000).unwrap(),
                busy: false
            }
        );
        assert_eq!(
            freebusy[2],
            EventInstance {
                start_time: DateTime::from_timestamp_millis(1584000000).unwrap(),
                end_time: DateTime::from_timestamp_millis(1593000000).unwrap(),
                busy: false
            }
        );
        assert_eq!(
            freebusy[3],
            EventInstance {
                start_time: DateTime::from_timestamp_millis(2188800000).unwrap(),
                end_time: DateTime::from_timestamp_millis(2197800000).unwrap(),
                busy: false
            }
        );
    }

    #[test]
    fn it_parses_intervals_for_rule() {
        let interval1 = ScheduleRuleInterval {
            start: Time {
                hours: 8,
                minutes: 30,
            },
            end: Time {
                hours: 9,
                minutes: 0,
            },
        };
        let interval2 = ScheduleRuleInterval {
            start: Time {
                hours: 10,
                minutes: 30,
            },
            end: Time {
                hours: 12,
                minutes: 30,
            },
        };
        let interval3 = ScheduleRuleInterval {
            start: Time {
                hours: 20,
                minutes: 30,
            },
            end: Time {
                hours: 21,
                minutes: 0,
            },
        };
        let interval4 = ScheduleRuleInterval {
            start: Time {
                hours: 20,
                minutes: 45,
            },
            end: Time {
                hours: 21,
                minutes: 50,
            },
        };
        let interval5 = ScheduleRuleInterval {
            start: Time {
                hours: 21,
                minutes: 50,
            },
            end: Time {
                hours: 22,
                minutes: 50,
            },
        };

        let mut rule = ScheduleRule {
            variant: ScheduleRuleVariant::WDay(Weekday::Mon),
            intervals: vec![
                interval2.clone(),
                interval1.clone(),
                interval3.clone(),
                interval4.clone(),
                interval5.clone(),
            ],
        };

        rule.parse_intervals();
        assert_eq!(rule.intervals.len(), 3);
        assert_eq!(
            rule.intervals,
            vec![
                interval1,
                interval2,
                ScheduleRuleInterval {
                    start: Time {
                        hours: interval3.start.hours,
                        minutes: interval3.start.minutes
                    },
                    end: Time {
                        hours: interval5.end.hours,
                        minutes: interval5.end.minutes
                    }
                },
            ]
        );
    }

    #[test]
    fn schedule_freebusy() {
        let schedule = Schedule::new(Default::default(), Default::default(), &chrono_tz::UTC);
        let timespan = TimeSpan::new(
            DateTime::from_timestamp_millis(1602108000000).unwrap(),
            DateTime::from_timestamp_millis(1602194400000).unwrap(),
        );
        let free = schedule.freebusy(&timespan);
        assert!(!free.is_empty());
    }

    #[test]
    fn schedule_freebusy_2() {
        let mut schedule = Schedule::new(Default::default(), Default::default(), &chrono_tz::UTC);
        schedule.rules = vec![
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ]
        .into_iter()
        .map(|wday| ScheduleRule {
            intervals: vec![ScheduleRuleInterval {
                start: Time {
                    hours: 0,
                    minutes: 0,
                },
                end: Time {
                    hours: 23,
                    minutes: 59,
                },
            }],
            variant: ScheduleRuleVariant::WDay(wday),
        })
        .collect::<Vec<_>>();

        // start -> 2021.4.1 at 00:00 and end -> 2021.5.1 at 00:00 in Europe/Oslo
        let timespan = TimeSpan::new(
            DateTime::from_timestamp_millis(1617228000000).unwrap(),
            DateTime::from_timestamp_millis(1617314400000).unwrap(),
        );
        let noon_utc = Utc.with_ymd_and_hms(2021, 4, 1, 0, 0, 0).unwrap();

        let free = schedule.freebusy(&timespan).inner();
        assert_eq!(free.len(), 2);
        assert_eq!(free[0].start_time, timespan.start());
        assert_eq!(
            free[0].end_time,
            noon_utc - TimeDelta::milliseconds(1000 * 60)
        ); // 23:59
        assert_eq!(free[1].start_time, noon_utc);
        assert_eq!(free[1].end_time, timespan.end());
    }
}

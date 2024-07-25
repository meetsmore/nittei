use std::{collections::HashMap, str::FromStr};

use chrono::{prelude::*, Duration};
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
        let min_date = self.timezone.ymd(now.year(), now.month(), now.day()) - Duration::days(2);
        let max_date = self.timezone.ymd(min_date.year() + 5, 1, 1);
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

// impl Time {
//     pub fn to_millis(&self, day: &Day, tzid: &Tz) -> i64 {
//         let dt = tzid.ymd(day.year, day.month, day.day).and_hms(0, 0, 0)
//             + Duration::minutes(self.minutes)
//             + Duration::hours(self.hours);
//         let millis = dt.timestamp_millis();
//         if dt.hour() != self.hours as u32 {
//             // DST probably
//             return millis - 1000 * 60 * 60 * 24;
//         }

//         millis
//     }
// }

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
        let mut hours = self.start.hours as u32;
        let date = tzid.ymd(day.year, day.month, day.day);

        // Try to find an hour in this day that is not invalid (DST) and greater than
        // or equals to `self.start.hour`
        let mut start = date.and_hms_opt(self.start.hours as u32, self.start.minutes as u32, 0);
        while start.is_none() {
            hours = (hours + 1) % 24;
            // Minutes can be zero now
            start = date.and_hms_opt(hours, 0, 0);
        }
        let start = start.unwrap();
        if self.start.hours as u32 > start.hour() {
            // No valid start for this date
            return None;
        }

        // Try to find an hour in this day that is not invalid (DST) and less than
        // or equals to `self.end.hour`
        let mut end = date.and_hms_opt(self.end.hours as u32, self.end.minutes as u32, 0);
        while end.is_none() {
            hours = if hours == 0 { 23 } else { hours - 1 };
            end = date.and_hms_opt(hours, self.end.minutes as u32, 0);
        }
        let end = end.unwrap();
        if end.hour() < self.end.hours as u32 {
            // No valid end hours for this date
            return None;
        }

        let start_ts = start.timestamp_millis();
        let end_ts = end.timestamp_millis();
        // Start should not be greater than end
        if start_ts > end_ts {
            return None;
        }

        Some(EventInstance {
            busy: false,
            start_ts,
            end_ts,
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

    pub fn date(&self, tzid: &Tz) -> Date<Tz> {
        tzid.ymd(self.year, self.month, self.day)
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
        let start = self.timezone.timestamp_millis(timespan.start());
        let end = self.timezone.timestamp_millis(timespan.end());

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

        let timespan = TimeSpan::new(0, 1000 * 60 * 60 * 24 * 30);
        let freebusy = schedule.freebusy(&timespan).inner();

        assert_eq!(freebusy.len(), 4);
        assert_eq!(
            freebusy[0],
            EventInstance {
                start_ts: 374400000,
                end_ts: 383400000,
                busy: false
            }
        );
        // Check that Date variant overrides wday variant
        assert_eq!(
            freebusy[1],
            EventInstance {
                start_ts: 982800000,
                end_ts: 995400000,
                busy: false
            }
        );
        assert_eq!(
            freebusy[2],
            EventInstance {
                start_ts: 1584000000,
                end_ts: 1593000000,
                busy: false
            }
        );
        assert_eq!(
            freebusy[3],
            EventInstance {
                start_ts: 2188800000,
                end_ts: 2197800000,
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
        let timespan = TimeSpan::new(1602108000000, 1602194400000);
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
        let timespan = TimeSpan::new(1617228000000, 1617314400000);
        let noon_utc = Utc.ymd(2021, 4, 1).and_hms(0, 0, 0).timestamp_millis();

        let free = schedule.freebusy(&timespan).inner();
        assert_eq!(free.len(), 2);
        assert_eq!(free[0].start_ts, timespan.start());
        assert_eq!(free[0].end_ts, noon_utc - 1000 * 60); // 23:59
        assert_eq!(free[1].start_ts, noon_utc);
        assert_eq!(free[1].end_ts, timespan.end());
    }
}

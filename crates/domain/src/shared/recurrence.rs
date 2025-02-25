use std::{cmp::Ordering, fmt::Display, str::FromStr};

use chrono::prelude::*;
use rrule::{Frequency, RRule, RRuleSet};
use serde::{Deserialize, Serialize, de::Visitor};
use thiserror::Error;
use ts_rs::TS;

use crate::CalendarSettings;

/// Frequency rule for recurring events
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum RRuleFrequency {
    Yearly,
    Monthly,
    Weekly,
    Daily,
}

/// Options for recurring events
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RRuleOptions {
    pub freq: RRuleFrequency,
    pub interval: isize,
    #[ts(optional)]
    pub count: Option<i32>,
    #[ts(optional)]
    pub until: Option<DateTime<Utc>>,
    #[ts(optional)]
    pub bysetpos: Option<Vec<isize>>,
    #[ts(optional)]
    pub byweekday: Option<Vec<WeekDayRecurrence>>,
    #[ts(optional)]
    pub bymonthday: Option<Vec<isize>>,
    #[ts(optional)]
    pub bymonth: Option<Vec<Month>>,
    #[ts(optional)]
    pub byyearday: Option<Vec<isize>>,
    #[ts(optional)]
    pub byweekno: Option<Vec<isize>>,
}

fn freq_convert(freq: &RRuleFrequency) -> Frequency {
    match freq {
        RRuleFrequency::Yearly => Frequency::Yearly,
        RRuleFrequency::Monthly => Frequency::Monthly,
        RRuleFrequency::Weekly => Frequency::Weekly,
        RRuleFrequency::Daily => Frequency::Daily,
    }
}

fn is_none_or_empty<T>(v: &Option<Vec<T>>) -> bool {
    !matches!(v, Some(v) if !v.is_empty())
}

impl RRuleOptions {
    pub fn is_valid(&self) -> bool {
        if let Some(count) = self.count {
            if !(1..740).contains(&count) {
                return false;
            }
        }
        if let Some(bysetpos) = &self.bysetpos {
            // Check that bysetpos is used with some other by* rule
            if !bysetpos.is_empty()
                && is_none_or_empty(&self.byweekday)
                && is_none_or_empty(&self.byweekno)
                && is_none_or_empty(&self.bymonth)
                && is_none_or_empty(&self.bymonthday)
                && is_none_or_empty(&self.byyearday)
            {
                // No other by* rule was specified
                return false;
            }
        }

        true
    }

    pub fn get_parsed_options(
        &self,
        start_time: DateTime<Utc>,
        calendar_settings: &CalendarSettings,
    ) -> anyhow::Result<RRuleSet> {
        let timezone = calendar_settings.timezone;
        let until = self.until.map(|u| u.with_timezone(&rrule::Tz::UTC));
        let dtstart = start_time.with_timezone(&rrule::Tz::Tz(timezone));

        let count = self.count.map(|c| std::cmp::max(c, 0) as u32);

        let mut bynweekday = Vec::new();
        if let Some(opts_byweekday) = &self.byweekday {
            for wday in opts_byweekday {
                match wday.nth() {
                    None => {
                        bynweekday.push(rrule::NWeekday::Nth(1, wday.weekday()));
                    }
                    Some(n) => {
                        bynweekday.push(rrule::NWeekday::Nth(n as i16, wday.weekday()));
                    }
                }
            }
        }

        let mut bymonthday = Vec::new();
        let mut bynmonthday = Vec::new();
        if let Some(opts_bymonthday) = &self.bymonthday {
            for monthday in opts_bymonthday {
                match monthday.cmp(&0) {
                    Ordering::Greater => bymonthday.push(monthday),
                    Ordering::Less => bynmonthday.push(monthday),
                    Ordering::Equal => {}
                }
            }
        }

        let mut rule = RRule::new(freq_convert(&self.freq))
            .by_month(&self.bymonth.clone().unwrap_or_default())
            .by_month_day(bymonthday.into_iter().map(|d| *d as i8).collect::<Vec<_>>())
            .by_weekday(bynweekday)
            .by_year_day(
                self.byyearday
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|d| d as i16)
                    .collect::<Vec<_>>(),
            )
            .by_set_pos(
                self.bysetpos
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|pos| pos as i32)
                    .collect::<Vec<_>>(),
            )
            .by_week_no(
                self.byweekno
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|w| w as i8)
                    .collect::<Vec<_>>(),
            )
            .by_hour(vec![dtstart.hour() as u8])
            .by_minute(vec![dtstart.minute() as u8])
            .by_second(vec![dtstart.second() as u8])
            .week_start(calendar_settings.week_start)
            .interval(self.interval as u16);

        if let Some(count) = count {
            rule = rule.count(count);
        }

        if let Some(until) = until {
            rule = rule.until(until);
        }

        Ok(rule.build(dtstart)?)
    }
}

impl Default for RRuleOptions {
    fn default() -> Self {
        Self {
            freq: RRuleFrequency::Daily,
            interval: 1,
            byweekday: None,
            bysetpos: None,
            count: None,
            until: None,
            bymonthday: None,
            bymonth: None,
            byyearday: None,
            byweekno: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, TS)]
#[ts(export, type = "string")]
pub struct WeekDayRecurrence {
    n: Option<isize>,
    weekday: Weekday,
}

impl WeekDayRecurrence {
    fn create(weekday: Weekday, n: Option<isize>) -> Option<Self> {
        if let Some(n) = n {
            if !Self::is_valid_n(n) {
                return None;
            }
        }
        Some(Self { n, weekday })
    }

    pub fn nth(&self) -> Option<isize> {
        self.n
    }
    pub fn weekday(&self) -> Weekday {
        self.weekday
    }

    pub fn new(weekday: Weekday) -> Option<Self> {
        Self::create(weekday, None)
    }

    pub fn new_nth(weekday: Weekday, n: isize) -> Option<Self> {
        Self::create(weekday, Some(n))
    }

    fn is_valid_n(n: isize) -> bool {
        n != 0 && n < 500 && n > -500
    }
}

impl Display for WeekDayRecurrence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n_prefix = match self.n {
            Some(n) => format!("{}", n),
            None => "".into(),
        };
        write!(f, "{}{}", n_prefix, self.weekday)
    }
}

#[derive(Error, Debug)]
pub enum InvalidWeekDayError {
    #[error("Invalid weekday specified: {0}")]
    InvalidWeekdayIdentifier(String),
    #[error("Malformed weekday: {0}")]
    Malformed(String),
}

impl FromStr for WeekDayRecurrence {
    type Err = InvalidWeekDayError;

    fn from_str(day: &str) -> Result<Self, Self::Err> {
        use InvalidWeekDayError::Malformed;

        let e = Malformed(day.to_string());
        match day.len() {
            0..=2 => Err(e),
            3 => {
                let wday = Weekday::from_str(day).map_err(|_| Malformed(day.to_string()))?;
                Ok(WeekDayRecurrence::new(wday).ok_or_else(|| Malformed(day.to_string()))?)
            }
            _ => {
                let wday = Weekday::from_str(&day[day.len() - 3..])
                    .map_err(|_| Malformed(day.to_string()))?;
                let n = day[0..day.len() - 3]
                    .parse::<isize>()
                    .map_err(|_| Malformed(day.to_string()))?;
                WeekDayRecurrence::new_nth(wday, n).ok_or(e)
            }
        }
    }
}

impl Serialize for WeekDayRecurrence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for WeekDayRecurrence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct WeekDayVisitor;

        impl Visitor<'_> for WeekDayVisitor {
            type Value = WeekDayRecurrence;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("A valid string representation of weekday")
            }

            fn visit_str<E>(self, value: &str) -> Result<WeekDayRecurrence, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse::<WeekDayRecurrence>()
                    .map_err(|_| E::custom(format!("Malformed weekday: {}", value)))
            }
        }

        deserializer.deserialize_str(WeekDayVisitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_valid_weekday_str_correctly() {
        assert_eq!(
            "mon".parse::<WeekDayRecurrence>().unwrap(),
            WeekDayRecurrence::new(Weekday::Mon).unwrap()
        );
        assert_eq!(
            "sun".parse::<WeekDayRecurrence>().unwrap(),
            WeekDayRecurrence::new(Weekday::Sun).unwrap()
        );
        assert_eq!(
            "1mon".parse::<WeekDayRecurrence>().unwrap(),
            WeekDayRecurrence::new_nth(Weekday::Mon, 1).unwrap()
        );
        assert_eq!(
            "17mon".parse::<WeekDayRecurrence>().unwrap(),
            WeekDayRecurrence::new_nth(Weekday::Mon, 17).unwrap()
        );
        assert_eq!(
            "170mon".parse::<WeekDayRecurrence>().unwrap(),
            WeekDayRecurrence::new_nth(Weekday::Mon, 170).unwrap()
        );
        assert_eq!(
            "+2mon".parse::<WeekDayRecurrence>().unwrap(),
            WeekDayRecurrence::new_nth(Weekday::Mon, 2).unwrap()
        );
        assert_eq!(
            "+22mon".parse::<WeekDayRecurrence>().unwrap(),
            WeekDayRecurrence::new_nth(Weekday::Mon, 22).unwrap()
        );
        assert_eq!(
            "-2mon".parse::<WeekDayRecurrence>().unwrap(),
            WeekDayRecurrence::new_nth(Weekday::Mon, -2).unwrap()
        );
        assert_eq!(
            "-22mon".parse::<WeekDayRecurrence>().unwrap(),
            WeekDayRecurrence::new_nth(Weekday::Mon, -22).unwrap()
        );
    }

    #[test]
    fn parses_invalid_weekday_str_correctly() {
        assert!("".parse::<WeekDayRecurrence>().is_err());
        assert!("-1".parse::<WeekDayRecurrence>().is_err());
        assert!("7".parse::<WeekDayRecurrence>().is_err());
        assert!("00".parse::<WeekDayRecurrence>().is_err());
        assert!("-1!?".parse::<WeekDayRecurrence>().is_err());
        assert!("-1WEDn".parse::<WeekDayRecurrence>().is_err());
        assert!("-1mond".parse::<WeekDayRecurrence>().is_err());
        assert!("mond".parse::<WeekDayRecurrence>().is_err());
        assert!("1000mon".parse::<WeekDayRecurrence>().is_err());
        assert!("0mon".parse::<WeekDayRecurrence>().is_err());
        assert!("000mon".parse::<WeekDayRecurrence>().is_err());
        assert!("+0mon".parse::<WeekDayRecurrence>().is_err());
    }

    #[test]
    fn serializes_weekday() {
        assert_eq!(
            WeekDayRecurrence::new(Weekday::Mon).unwrap().to_string(),
            "Mon"
        );
        assert_eq!(
            WeekDayRecurrence::new(Weekday::Tue).unwrap().to_string(),
            "Tue"
        );
        assert_eq!(
            WeekDayRecurrence::new(Weekday::Sun).unwrap().to_string(),
            "Sun"
        );
        assert_eq!(
            WeekDayRecurrence::new_nth(Weekday::Sun, 1)
                .unwrap()
                .to_string(),
            "1Sun"
        );
        assert_eq!(
            WeekDayRecurrence::new_nth(Weekday::Sun, -1)
                .unwrap()
                .to_string(),
            "-1Sun"
        );
    }
}

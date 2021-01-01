use crate::{calendar::domain::calendar_view::CalendarView, shared::entity::Entity};

use super::event_instance::EventInstance;
use chrono::{prelude::*, Duration};
use chrono_tz::Tz;
use rrule::{Frequenzy, ParsedOptions, RRule, RRuleSet};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RRuleFrequenzy {
    Yearly,
    Monthly,
    Weekly,
    Daily
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RRuleOptions {
    pub freq: RRuleFrequenzy,
    pub interval: isize,
    pub count: Option<i32>,
    pub until: Option<isize>,
    pub tzid: String,
    pub wkst: isize,
    pub bysetpos: Option<Vec<isize>>,
    pub byweekday: Option<Vec<isize>>,
    pub bynweekday: Option<Vec<Vec<isize>>>,
}
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub start_ts: i64,
    pub duration: i64,
    pub busy: bool,
    pub end_ts: Option<i64>,
    pub recurrence: Option<RRuleOptions>,
    pub exdates: Vec<i64>,
    pub calendar_id: String,
    pub user_id: String,
}

impl CalendarEvent {
    fn update_endtime(&mut self) {
        let opts = self.get_rrule_options();
        if (opts.count.is_some() && opts.count.unwrap() > 0) || opts.until.is_some() {
            let expand = self.expand(None);
            if let Some(last_occurence) = expand.last() {
                self.end_ts = Some(last_occurence.end_ts);
            } else {
                self.end_ts = None;
            }
        } else {
            self.end_ts = None;
        }
    }

    pub fn set_reccurrence(&mut self, reccurence: RRuleOptions, update_endtime: bool) {
        self.recurrence = Some(reccurence);
        if update_endtime {
            self.update_endtime();
        }
    }

    pub fn expand(&self, view: Option<&CalendarView>) -> Vec<EventInstance> {
        if self.recurrence.is_some() {
            let rrule_options = self.get_rrule_options();

            let tzid = rrule_options.tzid;
            let mut rrule_set = RRuleSet::new();
            for exdate in &self.exdates {
                let exdate = tzid.timestamp_millis(*exdate);
                rrule_set.exdate(exdate);
            }
            let rrule = RRule::new(rrule_options);
            rrule_set.rrule(rrule);

            let instances = match view {
                Some(view) => {
                    let view = view.as_datetime(&tzid);

                    // Also take the duration of events into consideration as the rrule library
                    // does not support duration on events.
                    let end = view.end - Duration::milliseconds(self.duration);

                    rrule_set.between(view.start, end, true)
                }
                None => rrule_set.all(),
            };

            instances
                .iter()
                .map(|occurence| {
                    let start_ts = occurence.timestamp_millis();

                    EventInstance {
                        start_ts,
                        end_ts: start_ts + self.duration,
                        busy: self.busy,
                    }
                })
                .collect()
        } else {
            vec![EventInstance {
                start_ts: self.start_ts,
                end_ts: self.start_ts + self.duration,
                busy: self.busy,
            }]
        }
    }

    fn freq_convert(freq: &RRuleFrequenzy) -> Frequenzy {
        match freq {
            RRuleFrequenzy::Yearly => Frequenzy::Yearly,
            RRuleFrequenzy::Monthly => Frequenzy::Monthly,
            RRuleFrequenzy::Weekly => Frequenzy::Weekly,
            RRuleFrequenzy::Daily => Frequenzy::Daily,
        }
    }

    fn get_rrule_options(&self) -> ParsedOptions {
        let options = self.recurrence.clone().unwrap();

        let tzid: Tz = options.tzid.parse().unwrap();
        let until = match options.until {
            Some(ts) => Some(tzid.timestamp(ts as i64 / 1000, 0)),
            None => None,
        };

        let dtstart = tzid.timestamp(self.start_ts / 1000, 0);

        let count = match options.count {
            Some(c) => Some(std::cmp::max(c, 0) as u32),
            None => None,
        };

        return ParsedOptions {
            freq: Self::freq_convert(&options.freq),
            count,
            bymonth: vec![],
            dtstart,
            byweekday: options.byweekday.unwrap_or_default().iter().map(|d| *d as usize).collect(),
            byhour: vec![dtstart.hour() as usize],
            bysetpos: options.bysetpos.unwrap_or_default(),
            byweekno: vec![],
            byminute: vec![dtstart.minute() as usize],
            bysecond: vec![dtstart.second() as usize],
            byyearday: vec![],
            bymonthday: vec![],
            bynweekday: options.bynweekday.clone().unwrap_or_default(),
            bynmonthday: vec![],
            until,
            wkst: options.wkst as usize,
            tzid,
            interval: options.interval as usize,
            byeaster: None,
        };
    }
}

impl Entity for CalendarEvent {
    fn id(&self) -> String {
        self.id.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono_tz::UTC;

    fn ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> DateTime<Tz> {
        UTC.ymd(year, month, day).and_hms(hour, minute, second)
    }

    #[test]
    fn daily_calendar_event() {
        let event = CalendarEvent {
            id: String::from("dsa"),
            start_ts: 1521317491239,
            busy: false,
            duration: 1000 * 60 * 60,
            recurrence: Some(RRuleOptions {
                freq: RRuleFrequenzy::Daily,
                interval: 1,
                tzid: UTC.to_string(),
                wkst: 0,
                until: None,
                count: Some(4),
                bynweekday: None,
                byweekday: None,
                bysetpos: None,
            }),
            end_ts: None,
            exdates: vec![1521317491239],
            calendar_id: String::from(""),
            user_id: String::from(""),
        };

        let oc = event.expand(None);
        println!("Occ: {:?}", oc);
        assert_eq!(oc.len(), 3);
    }
}

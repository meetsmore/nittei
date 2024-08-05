use std::collections::{HashMap, VecDeque};

use chrono::{prelude::*, TimeDelta};
use chrono_tz::Tz;
use date::format_date;
use serde::Serialize;

use crate::{date, event_instance::EventInstance, CompatibleInstances, ID};

#[derive(Serialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BookingSlot {
    pub start: DateTime<Utc>,
    pub duration: i64,
    pub available_until: DateTime<Utc>,
}

fn is_cursor_in_events(
    cursor: DateTime<Utc>,
    duration: i64,
    events: &CompatibleInstances,
) -> Option<&EventInstance> {
    events.as_ref().iter().find(|&event| {
        event.start_time <= cursor && event.end_time >= cursor + TimeDelta::milliseconds(duration)
    })
}

pub struct BookingSlotsOptions {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub duration: i64,
    pub interval: i64,
}

#[derive(Debug)]
pub struct UserFreeEvents {
    pub free_events: CompatibleInstances,
    pub user_id: ID,
}

#[derive(PartialEq, Debug)]
pub struct ServiceBookingSlot {
    pub start: DateTime<Utc>,
    pub duration: i64,
    pub user_ids: Vec<ID>,
}

#[derive(Debug)]
pub struct ServiceBookingSlots {
    pub dates: Vec<ServiceBookingSlotsDate>,
}

impl ServiceBookingSlots {
    pub fn new(slots: Vec<ServiceBookingSlot>, tz: Tz) -> Self {
        let mut slots = slots.into_iter().collect::<VecDeque<_>>();
        let mut dates = Vec::new();

        while !slots.is_empty() {
            dates.push(ServiceBookingSlotsDate::new(&mut slots, tz));
        }

        Self { dates }
    }
}

#[derive(Debug)]
pub struct ServiceBookingSlotsDate {
    pub date: String,
    pub slots: Vec<ServiceBookingSlot>,
}

impl ServiceBookingSlotsDate {
    pub fn new(slots: &mut VecDeque<ServiceBookingSlot>, tz: Tz) -> Self {
        assert!(!slots.is_empty());
        let first_date = slots[0].start;
        let date = format_date(&first_date);
        let mut date_slots = Vec::new();

        while let Some(current_date) = slots.front() {
            let current_date = format_date(&current_date.start);
            if current_date != date {
                break;
            } else {
                // Delete slot
                date_slots.push(slots.pop_front().unwrap());
            }
        }

        Self {
            date,
            slots: date_slots,
        }
    }
}

pub fn get_service_bookingslots(
    users_free: Vec<UserFreeEvents>,
    options: &BookingSlotsOptions,
) -> Vec<ServiceBookingSlot> {
    // Maybe use HashMap::with_capacity
    let mut slots_lookup: HashMap<i64, ServiceBookingSlot> = HashMap::new();

    for user in &users_free {
        let slots = get_booking_slots(&user.free_events, options);
        for slot in slots {
            if let Some(val) = slots_lookup.get(&slot.start.timestamp_millis()) {
                let mut user_ids = val.user_ids.clone();
                user_ids.push(user.user_id.clone());
                slots_lookup.insert(
                    slot.start.timestamp_millis(),
                    ServiceBookingSlot {
                        duration: slot.duration,
                        start: slot.start,
                        user_ids,
                    },
                );
            } else {
                slots_lookup.insert(
                    slot.start.timestamp_millis(),
                    ServiceBookingSlot {
                        duration: slot.duration,
                        start: slot.start,
                        user_ids: vec![user.user_id.clone()],
                    },
                );
            }
        }
    }

    let mut slots = slots_lookup.drain().map(|s| s.1).collect::<Vec<_>>();
    slots.sort_by_key(|s| s.start);

    slots
}

pub fn get_booking_slots(
    free_events: &CompatibleInstances,
    options: &BookingSlotsOptions,
) -> Vec<BookingSlot> {
    let mut booking_slots = Vec::new();
    let &BookingSlotsOptions {
        start_time,
        end_time,
        duration,
        interval,
    } = options;

    if duration < 1 {
        return booking_slots;
    }

    let mut cursor = start_time;
    while cursor + TimeDelta::milliseconds(interval) <= end_time {
        let available_event = is_cursor_in_events(cursor, duration, free_events);
        if let Some(event) = available_event {
            booking_slots.push(BookingSlot {
                start: cursor,
                duration,
                available_until: event.end_time,
            });
        }

        cursor += TimeDelta::milliseconds(interval)
    }

    booking_slots
}

pub fn validate_slots_interval(interval: i64) -> bool {
    let min_interval = 1000 * 60 * 5;
    let max_interval = 1000 * 60 * 60 * 2;
    interval >= min_interval && interval <= max_interval
}

pub struct BookingSlotsQuery {
    pub start_date: String,
    pub end_date: String,
    pub timezone: Option<Tz>,
    pub duration: i64,
    pub interval: i64,
}

pub enum BookingQueryError {
    InvalidInterval,
    InvalidDate(String),
    InvalidTimespan,
}

pub struct BookingTimespan {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

pub fn validate_bookingslots_query(
    query: &BookingSlotsQuery,
) -> Result<BookingTimespan, BookingQueryError> {
    if !validate_slots_interval(query.interval) {
        return Err(BookingQueryError::InvalidInterval);
    }

    let tz = query.timezone.unwrap_or(chrono_tz::UTC);

    let parsed_start_date = match date::is_valid_date(&query.start_date) {
        Ok(val) => val,
        Err(_) => return Err(BookingQueryError::InvalidDate(query.start_date.clone())),
    };
    let parsed_end_date = match date::is_valid_date(&query.end_date) {
        Ok(val) => val,
        Err(_) => return Err(BookingQueryError::InvalidDate(query.end_date.clone())),
    };

    let start_date = tz
        .with_ymd_and_hms(
            parsed_start_date.0,
            parsed_start_date.1,
            parsed_start_date.2,
            0,
            0,
            0,
        )
        .unwrap();
    let start_time = start_date
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap()
        .with_timezone(&Utc);
    let end_date = tz
        .with_ymd_and_hms(
            parsed_end_date.0,
            parsed_end_date.1,
            parsed_end_date.2,
            0,
            0,
            0,
        )
        .unwrap();
    let end_time = end_date
        .with_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap()
        .with_timezone(&Utc)
        + TimeDelta::milliseconds(1000 * 60 * 60 * 24);

    Ok(BookingTimespan {
        start_time,
        end_time,
    })
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn get_booking_slots_empty() {
        let slots = get_booking_slots(
            &CompatibleInstances::new(Vec::new()),
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(0).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );
        assert!(slots.is_empty());
    }

    #[test]
    fn get_booking_slots_from_one_event_1() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(2).unwrap(),
            end_time: DateTime::from_timestamp_millis(12).unwrap(),
        };

        let slots = get_booking_slots(
            &CompatibleInstances::new(vec![e1]),
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(0).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );

        assert!(slots.is_empty());
    }

    #[test]
    fn get_booking_slots_from_one_event_2() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(2).unwrap(),
            end_time: DateTime::from_timestamp_millis(22).unwrap(),
        };

        let slots = get_booking_slots(
            &CompatibleInstances::new(vec![e1]),
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(0).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );

        assert_eq!(slots.len(), 1);
        assert_eq!(
            slots[0],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(22).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(10).unwrap()
            }
        );
    }

    #[test]
    fn get_booking_slots_from_one_event_3() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(2).unwrap(),
            end_time: DateTime::from_timestamp_millis(42).unwrap(),
        };

        let slots = get_booking_slots(
            &CompatibleInstances::new(vec![e1]),
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(0).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );

        assert_eq!(slots.len(), 3);
        assert_eq!(
            slots[0],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(42).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(10).unwrap()
            }
        );
        assert_eq!(
            slots[1],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(42).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(20).unwrap()
            }
        );
        assert_eq!(
            slots[2],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(42).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(30).unwrap()
            }
        );
    }

    #[test]
    fn get_booking_slots_from_two_events() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(0).unwrap(),
            end_time: DateTime::from_timestamp_millis(22).unwrap(),
        };

        let e2 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(30).unwrap(),
            end_time: DateTime::from_timestamp_millis(50).unwrap(),
        };

        let slots = get_booking_slots(
            &CompatibleInstances::new(vec![e1, e2]),
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(0).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );

        assert_eq!(slots.len(), 4);
        assert_eq!(
            slots[0],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(22).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(0).unwrap()
            }
        );
        assert_eq!(
            slots[1],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(22).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(10).unwrap()
            }
        );
        assert_eq!(
            slots[2],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(50).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(30).unwrap()
            }
        );
        assert_eq!(
            slots[3],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(50).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(40).unwrap()
            }
        );
    }

    #[test]
    fn get_booking_slots_from_many_events() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(0).unwrap(),
            end_time: DateTime::from_timestamp_millis(2).unwrap(),
        };

        let e2 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(33).unwrap(),
            end_time: DateTime::from_timestamp_millis(50).unwrap(),
        };

        let e3 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(80).unwrap(),
            end_time: DateTime::from_timestamp_millis(90).unwrap(),
        };

        let e4 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(90).unwrap(),
            end_time: DateTime::from_timestamp_millis(100).unwrap(),
        };

        let e5 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(99).unwrap(),
            end_time: DateTime::from_timestamp_millis(120).unwrap(),
        };

        let e6 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(140).unwrap(),
            end_time: DateTime::from_timestamp_millis(160).unwrap(),
        };
        let availability = CompatibleInstances::new(vec![e1, e3, e4, e2, e6, e5]);

        let slots = get_booking_slots(
            &availability,
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(0).unwrap(),
                end_time: DateTime::from_timestamp_millis(99).unwrap(),
                duration: 10,
                interval: 10,
            },
        );

        assert_eq!(slots.len(), 2);
        assert_eq!(
            slots[0],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(50).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(40).unwrap()
            }
        );
        assert_eq!(
            slots[1],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(120).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(80).unwrap()
            }
        );
    }

    #[test]
    fn slot_that_fits_right_at_end() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(81).unwrap(),
            end_time: DateTime::from_timestamp_millis(100).unwrap(),
        };

        let slots = get_booking_slots(
            &CompatibleInstances::new(vec![e1]),
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(0).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );

        assert_eq!(slots.len(), 1);
        assert_eq!(
            slots[0],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(90).unwrap()
            }
        );
    }

    #[test]
    fn slot_that_crosses_end() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(81).unwrap(),
            end_time: DateTime::from_timestamp_millis(120).unwrap(),
        };

        let slots = get_booking_slots(
            &CompatibleInstances::new(vec![e1]),
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(0).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );

        assert_eq!(slots.len(), 1);
        assert_eq!(
            slots[0],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(120).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(90).unwrap()
            }
        );
    }

    #[test]
    fn slot_that_crosses_start() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(2).unwrap(),
            end_time: DateTime::from_timestamp_millis(30).unwrap(),
        };

        let slots = get_booking_slots(
            &CompatibleInstances::new(vec![e1]),
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(10).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );

        assert_eq!(slots.len(), 2);
        assert_eq!(
            slots[0],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(30).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(10).unwrap()
            }
        );
        assert_eq!(
            slots[1],
            BookingSlot {
                available_until: DateTime::from_timestamp_millis(30).unwrap(),
                duration: 10,
                start: DateTime::from_timestamp_millis(20).unwrap()
            }
        );
    }

    #[test]
    fn generate_service_bookingslots_with_one_user_in_service() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(2).unwrap(),
            end_time: DateTime::from_timestamp_millis(30).unwrap(),
        };

        let user_id = ID::default();

        let users_free = vec![UserFreeEvents {
            free_events: CompatibleInstances::new(vec![e1]),
            user_id: user_id.clone(),
        }];

        let slots = get_service_bookingslots(
            users_free,
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(10).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );

        assert_eq!(slots.len(), 2);
        assert_eq!(
            slots[0],
            ServiceBookingSlot {
                duration: 10,
                start: DateTime::from_timestamp_millis(10).unwrap(),
                user_ids: vec![user_id.clone()]
            }
        );
        assert_eq!(
            slots[1],
            ServiceBookingSlot {
                duration: 10,
                start: DateTime::from_timestamp_millis(20).unwrap(),
                user_ids: vec![user_id.clone()]
            }
        );
    }

    #[test]
    fn generate_service_bookingslots_with_two_users_in_service() {
        let e1 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(2).unwrap(),
            end_time: DateTime::from_timestamp_millis(30).unwrap(),
        };

        let e2 = EventInstance {
            busy: false,
            start_time: DateTime::from_timestamp_millis(33).unwrap(),
            end_time: DateTime::from_timestamp_millis(52).unwrap(),
        };

        let user_id_1 = ID::default();
        let user_id_2 = ID::default();
        let users_free = vec![
            UserFreeEvents {
                free_events: CompatibleInstances::new(vec![e1.clone()]),
                user_id: user_id_1.clone(),
            },
            UserFreeEvents {
                free_events: CompatibleInstances::new(vec![e1, e2]),
                user_id: user_id_2.clone(),
            },
        ];

        let slots = get_service_bookingslots(
            users_free,
            &BookingSlotsOptions {
                start_time: DateTime::from_timestamp_millis(10).unwrap(),
                end_time: DateTime::from_timestamp_millis(100).unwrap(),
                duration: 10,
                interval: 10,
            },
        );
        assert_eq!(slots.len(), 3);
        assert_eq!(
            slots[0],
            ServiceBookingSlot {
                duration: 10,
                start: DateTime::from_timestamp_millis(10).unwrap(),
                user_ids: vec![user_id_1.clone(), user_id_2.clone()]
            }
        );
        assert_eq!(
            slots[1],
            ServiceBookingSlot {
                duration: 10,
                start: DateTime::from_timestamp_millis(20).unwrap(),
                user_ids: vec![user_id_1.clone(), user_id_2.clone()]
            }
        );
        assert_eq!(
            slots[2],
            ServiceBookingSlot {
                duration: 10,
                start: DateTime::from_timestamp_millis(40).unwrap(),
                user_ids: vec![user_id_2.clone()]
            }
        );
    }

    #[test]
    fn groups_bookingslots_by_date() {
        // Case 1
        let slots = Vec::default();
        let grouped_slots = ServiceBookingSlots::new(slots, chrono_tz::UTC);
        assert!(grouped_slots.dates.is_empty());

        // Case 2
        let user_id = ID::default();
        let slots = vec![ServiceBookingSlot {
            duration: 1000 * 60 * 15,
            start: DateTime::from_timestamp_millis(0).unwrap(),
            user_ids: vec![user_id],
        }];

        let grouped_slots = ServiceBookingSlots::new(slots, chrono_tz::UTC);
        assert_eq!(grouped_slots.dates.len(), 1);
        assert_eq!(grouped_slots.dates[0].slots.len(), 1);

        // Case 3
        let intervals = vec![1, 5, 10, 15, 20, 30, 60, 120, 360];
        for interval in intervals {
            let user_id = ID::default();
            let mut slots = Vec::default();
            let slots_interval = 1000 * 60 * interval;
            let slots_per_day = 1000 * 60 * 60 * 24 / slots_interval;
            let number_of_days = 5;

            for i in 0..slots_per_day * number_of_days {
                slots.push(ServiceBookingSlot {
                    duration: 1000 * 60 * interval,
                    start: DateTime::from_timestamp_millis(i * 1000 * 60 * interval).unwrap(),
                    user_ids: vec![user_id.clone()],
                });
            }

            let grouped_slots = ServiceBookingSlots::new(slots, chrono_tz::UTC);
            assert_eq!(grouped_slots.dates.len() as i64, number_of_days);
            for date_slots in grouped_slots.dates {
                assert_eq!(date_slots.slots.len() as i64, slots_per_day);
            }
        }
    }
}

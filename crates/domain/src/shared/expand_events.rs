use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tracing::warn;

use crate::{Calendar, CalendarEvent, EventInstance, TimeSpan, ID};

/// Generate a map of recurring_event_id to original_start_time
/// This is used to remove exceptions from the expanded events
/// The key is the recurring_event_id (as string) and the value is a vector of original_start_time
pub fn generate_map_exceptions_start_times(
    events: &Vec<CalendarEvent>,
) -> HashMap<ID, Vec<DateTime<Utc>>> {
    let mut map_recurring_event_id_to_exceptions = HashMap::new();
    for event in events {
        if let Some(recurring_event_id) = &event.recurring_event_id {
            if let Some(original_start_time) = &event.original_start_time {
                map_recurring_event_id_to_exceptions
                    .entry(recurring_event_id.clone())
                    .or_insert_with(Vec::new)
                    .push(*original_start_time);
            } else {
                warn!(
                    "Event with id: {} has a recurring_event_id but no original_start_time",
                    event.id
                );
            }
        }
    }

    map_recurring_event_id_to_exceptions
}

/// Expand an event received
/// This function will expand the event received and return a vector of EventInstance
/// This function will also remove exceptions from the expanded events
pub fn expand_event_and_remove_exceptions(
    calendar: &Calendar,
    event: &CalendarEvent,
    exceptions: &[DateTime<Utc>],
    timespan: &TimeSpan,
) -> anyhow::Result<Vec<EventInstance>> {
    let expanded_events = event.expand(Some(timespan), &calendar.settings)?;

    // If we have exceptions, remove them from the expanded events
    let expanded_events = if !exceptions.is_empty() {
        event.remove_changed_instances(expanded_events, exceptions)
    } else {
        expanded_events
    };

    Ok(expanded_events)
}

/// Expand all events received
/// This function will expand all events received and return a vector of EventInstance
/// This function will also remove exceptions from the expanded events
pub fn expand_all_events_and_remove_exceptions(
    calendars: &HashMap<String, &Calendar>,
    events: &Vec<CalendarEvent>,
    timespan: &TimeSpan,
) -> anyhow::Result<Vec<EventInstance>> {
    let map_recurring_event_id_to_exceptions = generate_map_exceptions_start_times(events);

    let mut all_expanded_events = Vec::new();
    // For each event, expand it and add the instances to the all_expanded_events
    for event in events {
        let calendar = calendars
            .get(&event.calendar_id.to_string())
            .ok_or_else(|| anyhow::anyhow!("Calendar with id: {} not found", event.calendar_id))?;
        let expanded_events = event.expand(Some(timespan), &calendar.settings)?;

        // Get the exceptions for this event
        let exceptions = map_recurring_event_id_to_exceptions.get(&event.id);

        // If we have exceptions, remove them from the expanded events
        let expanded_events = if let Some(exceptions) = exceptions {
            event.remove_changed_instances(expanded_events, exceptions)
        } else {
            expanded_events
        };

        all_expanded_events.extend(expanded_events);
    }

    Ok(all_expanded_events)
}

#[cfg(test)]
mod test {
    use chrono::Utc;

    use super::expand_event_and_remove_exceptions;
    use crate::{generate_map_exceptions_start_times, Calendar, CalendarEvent, TimeSpan, ID};

    #[test]
    fn test_generate_map_exceptions_start_times() {
        let id = ID::default();
        let id2 = ID::default();
        let id3 = ID::default();

        let recurring_event_id = ID::default();
        let recurring_event_id2 = ID::default();

        let events = vec![
            CalendarEvent {
                id: recurring_event_id.clone(),
                ..Default::default()
            },
            CalendarEvent {
                id: recurring_event_id2.clone(),
                ..Default::default()
            },
            CalendarEvent {
                id: id.clone(),
                recurring_event_id: Some(recurring_event_id.clone()),
                original_start_time: Some(Utc::now()),
                ..Default::default()
            },
            CalendarEvent {
                id: id2.clone(),
                recurring_event_id: Some(recurring_event_id.clone()),
                original_start_time: Some(Utc::now()),
                ..Default::default()
            },
            CalendarEvent {
                id: id3.clone(),
                recurring_event_id: Some(recurring_event_id2.clone()),
                original_start_time: Some(
                    Utc::now()
                        .checked_add_signed(chrono::Duration::days(1))
                        .unwrap(),
                ),
                ..Default::default()
            },
        ];

        let map = generate_map_exceptions_start_times(&events);

        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&recurring_event_id).unwrap().len(), 2);
        assert_eq!(map.get(&recurring_event_id2).unwrap().len(), 1);
    }

    #[test]
    fn test_generate_map_exceptions_start_times_no_original_start_time() {
        let id = ID::default();
        let id2 = ID::default();
        let id3 = ID::default();

        let recurring_event_id = ID::default();
        let recurring_event_id2 = ID::default();

        let events = vec![
            CalendarEvent {
                id: recurring_event_id.clone(),
                ..Default::default()
            },
            CalendarEvent {
                id: recurring_event_id2.clone(),
                ..Default::default()
            },
            CalendarEvent {
                id: id.clone(),
                recurring_event_id: Some(recurring_event_id.clone()),
                ..Default::default()
            },
            CalendarEvent {
                id: id2.clone(),
                recurring_event_id: Some(recurring_event_id.clone()),
                ..Default::default()
            },
            CalendarEvent {
                id: id3.clone(),
                recurring_event_id: Some(recurring_event_id2.clone()),
                ..Default::default()
            },
        ];

        let map = generate_map_exceptions_start_times(&events);

        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_expand_event_and_remove_exceptions_for_normal_event() {
        let calendar = Calendar {
            id: ID::default(),
            ..Default::default()
        };

        let event = CalendarEvent {
            id: ID::default(),
            ..Default::default()
        };

        let exceptions = vec![];

        let timespan = TimeSpan::new(
            Utc::now(),
            Utc::now()
                .checked_add_signed(chrono::Duration::days(1))
                .unwrap(),
        );

        let instances =
            expand_event_and_remove_exceptions(&calendar, &event, exceptions.as_slice(), &timespan)
                .unwrap();

        assert_eq!(instances.len(), 1);

        let instance = &instances[0];
        assert_eq!(instance.start_time, event.start_time);
        assert_eq!(instance.end_time, event.end_time);
    }

    #[test]
    fn test_expand_event_and_remove_exceptions_for_exception_event() {
        let calendar = Calendar {
            id: ID::default(),
            ..Default::default()
        };

        let now = Utc::now();
        let event = CalendarEvent {
            id: ID::default(),
            start_time: now,
            duration: 1000 * 60 * 60,
            ..Default::default()
        };

        let timespan = TimeSpan::new(
            now,
            now.checked_add_signed(chrono::Duration::days(1)).unwrap(),
        );

        let instances =
            expand_event_and_remove_exceptions(&calendar, &event, vec![now].as_slice(), &timespan)
                .unwrap();

        assert_eq!(instances.len(), 0);
    }
}

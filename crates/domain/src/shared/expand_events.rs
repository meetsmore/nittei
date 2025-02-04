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

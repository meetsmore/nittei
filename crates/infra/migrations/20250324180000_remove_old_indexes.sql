DROP INDEX IF EXISTS event_metadata;

DROP INDEX IF EXISTS events_parent_id;

DROP INDEX IF EXISTS calendar_events__calendar_uid__start_time__recurring_until__not_null_recurrence;

DROP INDEX IF EXISTS calendar_events__recurring_event_uid;

DROP INDEX IF EXISTS calendar_events__event_type_status_start_time;

DROP INDEX IF EXISTS calendar_events__calendar_uid__start_time__end_time_idx;

DROP INDEX IF EXISTS calendar_events__calendar_uid_event_type_status_start_time;

DROP INDEX IF EXISTS calendar_events__calendar_uid_idx;
-- Add a compound indexes for calendar_uid, start_time and end_time
-- This index is useful for querying events that have no recurrence
CREATE INDEX IF NOT EXISTS calendar_events__calendar_uid__start_time__end_time_idx ON calendar_events (calendar_uid, start_time, end_time);

-- Add a compound indexes for calendar_uid, start_time, only if recurrence is not null
-- This index is useful for querying events that have recurrence
CREATE INDEX IF NOT EXISTS calendar_events__calendar_uid__start_time__not_null_recurrence_idx ON calendar_events (calendar_uid, start_time)
WHERE
  recurrence::text <> 'null';
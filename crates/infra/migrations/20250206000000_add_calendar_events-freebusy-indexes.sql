-- Add a compound indexes for calendar_uid, start_time and end_time
-- This index is useful for querying events that have no recurrence
CREATE INDEX IF NOT EXISTS calendar_events__calendar_uid__start_time__end_time_idx ON calendar_events (calendar_uid, start_time, end_time);

-- Add a compound indexes for calendar_uid, start_time, only if recurrence is not null (JSON null)
-- This index is useful for querying events that have recurrence
-- We do it this way to avoid having a GIN index on the recurrence column, which is more heavy and not needed here
CREATE INDEX IF NOT EXISTS calendar_events__calendar_uid__start_time__not_null_recurrence_idx ON calendar_events (calendar_uid, start_time)
WHERE
  recurrence::text <> 'null';
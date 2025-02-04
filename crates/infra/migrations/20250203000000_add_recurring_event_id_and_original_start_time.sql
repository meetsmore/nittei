-- Add recurring_event_id and original_start_time to events
ALTER TABLE
  calendar_events
ADD
  COLUMN recurring_event_uid uuid,
ADD
  COLUMN original_start_time TIMESTAMPTZ;

-- Add a compound indexes for recurring_event_id and original_start_time
CREATE INDEX IF NOT EXISTS calendar_events__recurring_event_id__original_start_time_idx ON calendar_events (recurring_event_uid, original_start_time);
-- Add index on recurring events with (or without) recurring_until
-- In all cases, they have a recurrence_jsonb
CREATE INDEX IF NOT EXISTS calendar_events__recurring_range_partial ON calendar_events (user_uid, start_time, recurring_until)
WHERE
  recurrence_jsonb IS NOT NULL;

-- Add index on non-recurring events
-- This excludes events that have a recurrence_jsonb and an original_start_time (ex: recurring events and their exceptions)
CREATE INDEX IF NOT EXISTS calendar_events__non_recurring_events_partial ON calendar_events (user_uid, start_time, end_time, status, busy)
WHERE
  recurrence_jsonb IS NULL
  AND original_start_time IS NULL;
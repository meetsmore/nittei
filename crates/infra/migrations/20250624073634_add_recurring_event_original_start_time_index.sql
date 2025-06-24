-- Add index on recurring events with (or without) recurring_until
-- In all cases, they have a recurrence_jsonb
CREATE INDEX IF NOT EXISTS calendar_events__recurring_range_partial ON calendar_events (user_uid, start_time, recurring_until)
WHERE
  recurrence_jsonb IS NOT NULL;
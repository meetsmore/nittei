-- Create an index on `calendar_uid` + `event_type` + `status` + `start_time` in the `calendar_events` table.
-- Useful for querying on specific users
CREATE INDEX IF NOT EXISTS calendar_events__calendar_uid_event_type_status_start_time ON calendar_events (calendar_uid, event_type, status, start_time);

-- Create an index on `event_type` + `status` + `start_time` in the `calendar_events` table.
-- Useful for querying on the whole account
CREATE INDEX IF NOT EXISTS calendar_events__event_type_status_start_time ON calendar_events (event_type, status, start_time);

-- Create an index on `calendar_uid` + `original_start_time` in the `calendar_events` table.
-- Useful for querying on specific users
CREATE INDEX IF NOT EXISTS calendar_events__calendar_uid_original_start_time ON calendar_events (calendar_uid, original_start_time);

-- Create an index on `recurring_event_uid` in the `calendar_events` table.
-- Useful for searching the exceptions of a recurring event
CREATE INDEX IF NOT EXISTS calendar_events__recurring_event_uid ON calendar_events (recurring_event_uid);

-- Create an index on `calendar_uid` + `created` in the `calendar_events` table.
-- Useful for querying on specific users
CREATE INDEX IF NOT EXISTS calendar_events__calendar_uid_created ON calendar_events (calendar_uid, created);

-- Create an index on `updated` in the `calendar_events` table.
-- Useful for querying on the whole account (all updated between a range)
CREATE INDEX IF NOT EXISTS calendar_events__updated ON calendar_events (updated);
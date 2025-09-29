-- Create compound index on `account_uid`, `status`, `event_type`, `start_time`
-- Useful for cross-account queries
CREATE INDEX IF NOT EXISTS calendar_events__account_uid__status__event_type__start_time ON calendar_events(account_uid, status, event_type, start_time);
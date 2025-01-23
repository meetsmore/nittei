-- Add an index on `calendar_uid` in `events_groups`
CREATE INDEX IF NOT EXISTS events_groups__calendar_uid_idx ON events_groups (calendar_uid);

-- Add an index on `calendar_uid` in `calendar_events`
CREATE INDEX IF NOT EXISTS calendar_events__calendar_uid_idx ON calendar_events (calendar_uid);

-- Add an index on `account_uid` in `users`
CREATE INDEX IF NOT EXISTS users__account_uid_idx ON users (account_uid);
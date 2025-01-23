-- Add an index on `calendar_uid` in `events_groups`
CREATE INDEX IF NOT EXISTS events_groups__calendar_uid_idx ON events_groups (calendar_uid);
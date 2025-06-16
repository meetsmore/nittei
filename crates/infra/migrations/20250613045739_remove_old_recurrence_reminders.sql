-- Create compound index on `user_uid`, `start_time` where `recurrence_jsonb` is not null (recurring gcal events)
CREATE INDEX IF NOT EXISTS calendar_events__user_uid__start_time__not_null_recurrence_jsonb ON calendar_events(user_uid, start_time)
WHERE
  recurrence_jsonb IS NOT NULL;

-- Create compound index on `user_uid`, `event_type` where `recurrence_jsonb` is not null (block events)
CREATE INDEX IF NOT EXISTS calendar_events__user_uid__event_type__not_null_recurrence_jsonb ON calendar_events(user_uid, event_type)
WHERE
  recurrence_jsonb IS NOT NULL;

-- Delete compound index on `user_uid`, `start_time` where `recurrence` is not null (recurring gcal events)
DROP INDEX IF EXISTS calendar_events__user_uid__start_time__not_null_recurrence;

-- Delete compound index on `user_uid`, `event_type` where `recurrence` is not null (block events)
DROP INDEX IF EXISTS calendar_events__user_uid__event_type__not_null_recurrence;

-- Remove `recurrence` and `reminders` columns from `calendar_events` table.
-- We now use `recurrence_jsonb` and `reminders_jsonb` instead.
ALTER TABLE
  "calendar_events" DROP COLUMN IF EXISTS "recurrence";

ALTER TABLE
  "calendar_events" DROP COLUMN IF EXISTS "reminders";
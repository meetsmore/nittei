-- Remove index on `metadata` column in `calendar_events` table
DROP INDEX IF EXISTS event_metadata;
-- Rename "idx_calendar_events_external_id" to "calendar_events__external_id"
DO $$ BEGIN IF NOT EXISTS (
  SELECT 1
  FROM pg_indexes
  WHERE indexname = 'calendar_events__external_id'
) THEN EXECUTE 'ALTER INDEX idx_calendar_events_external_id RENAME TO calendar_events__external_id';
END IF;
END $$;
-- DELETE "events_parent_id" and create compound index "calendar_events__external_parent_id__status__event_type"
DROP INDEX IF EXISTS events_parent_id;
CREATE INDEX calendar_events__external_parent_id__status__event_type ON calendar_events(external_parent_id, status, event_type);
-- DELETE "calendar_events__calendar_uid__start_time__not_null_recurrence_" and create compound index "calendar_events__user_uid__start_time__not_null_recurrence"
DROP INDEX IF EXISTS calendar_events__calendar_uid__start_time__recurring_until__not_null_recurrence;
CREATE INDEX calendar_events__user_uid__start_time__not_null_recurrence ON calendar_events(user_uid, start_time)
WHERE recurrence IS NOT NULL;
-- Create another index for recurring events based on the type
CREATE INDEX calendar_events__user_uid__event_type__not_null_recurrence ON calendar_events(user_uid, event_type)
WHERE recurrence IS NOT NULL;
-- Remove "calendar_events__recurring_event_uid" index, as it's already handled by "calendar_events__recurring_event_id__original_start_time_idx"
DROP INDEX IF EXISTS calendar_events__recurring_event_uid;
-- Remove "calendar_events__calendar_uid_original_start_time" index, as it's already handled by "calendar_events__recurring_event_id__original_start_time_idx"
-- (not exactly the same, but we can assume that recurring_event_uid will be used in the same queries as "original_start_time")
DROP INDEX IF EXISTS calendar_events__calendar_uid_original_start_time;
-- Above is ok
-- Recreate "calendar_events__event_type_status_start_time" so that it includes "account_uid" column, but uses "end_time" instead of "start_time"
DROP INDEX IF EXISTS calendar_events__event_type_status_start_time;
CREATE INDEX calendar_events__account_uid__status__event_type__end_time ON calendar_events(account_uid, status, event_type, end_time);
-- Delete "calendar_events__calendar_uid__start_time__end_time_idx" and "calendar_events__calendar_uid_event_type_status_start_time"
DROP INDEX IF EXISTS calendar_events__calendar_uid__start_time__end_time_idx;
DROP INDEX IF EXISTS calendar_events__calendar_uid_event_type_status_start_time;
-- Create an index on `user_uid`, `startTime`, `endTime`, `status` columns in `calendar_events` table
CREATE INDEX calendar_events__user_uid__start_time__end_time__status ON calendar_events(user_uid, start_time, end_time, status);
-- Create an index on `user_uid`, `status` and `event_type` columns in `calendar_events` table
CREATE INDEX calendar_events__user_uid__status__event_type ON calendar_events(user_uid, status, event_type);
-- Delete `calendar_events__calendar_uid_idx` as we have `calendar_events__calendar_uid_created` index
DROP INDEX IF EXISTS calendar_events__calendar_uid_idx;
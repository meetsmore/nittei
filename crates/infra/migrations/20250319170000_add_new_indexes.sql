-- Rename "idx_calendar_events_external_id" to "calendar_events__external_id"
-- Useful for directly querying events by external id
DO $$ BEGIN IF NOT EXISTS (
  SELECT
    1
  FROM
    pg_indexes
  WHERE
    indexname = 'calendar_events__external_id'
) THEN EXECUTE 'ALTER INDEX idx_calendar_events_external_id RENAME TO calendar_events__external_id';

END IF;

END $$;

-- Create compound index on `external_parent_id`, `status`, `event_type`
-- Useful for directly querying events by parent id
CREATE INDEX IF NOT EXISTS calendar_events__external_parent_id__status__event_type ON calendar_events(external_parent_id, status, event_type);

-- Create compound index on `user_uid`, `start_time` where `recurrence` is not null (recurring gcal events)
CREATE INDEX IF NOT EXISTS calendar_events__user_uid__start_time__not_null_recurrence ON calendar_events(user_uid, start_time)
WHERE
  recurrence IS NOT NULL;

-- Create compound index on `user_uid`, `event_type` where `recurrence` is not null (block events)
CREATE INDEX IF NOT EXISTS calendar_events__user_uid__event_type__not_null_recurrence ON calendar_events(user_uid, event_type)
WHERE
  recurrence IS NOT NULL;

-- Create compound index on `account_uid`, `status`, `event_type`, `end_time`
-- Useful for cross-account queries
CREATE INDEX IF NOT EXISTS calendar_events__account_uid__status__event_type__end_time ON calendar_events(account_uid, status, event_type, end_time);

-- Create compound index on `user_uid`, `start_time`, `end_time`, `status`
-- Useful for single-account queries
CREATE INDEX IF NOT EXISTS calendar_events__user_uid__start_time__end_time__status ON calendar_events(user_uid, start_time, end_time, status);

-- Create an index on `user_uid`, `status` and `event_type` columns in `calendar_events` table
-- Useful for single-account queries
CREATE INDEX IF NOT EXISTS calendar_events__user_uid__status__event_type ON calendar_events(user_uid, status, event_type);
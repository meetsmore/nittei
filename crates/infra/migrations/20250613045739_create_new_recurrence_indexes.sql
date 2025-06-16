-- Create compound index on `user_uid`, `start_time` where `recurrence_jsonb` is not null (recurring gcal events)
CREATE INDEX IF NOT EXISTS calendar_events__user_uid__start_time__not_null_recurrence_jsonb ON calendar_events(user_uid, start_time)
WHERE
  recurrence_jsonb IS NOT NULL;

-- Create compound index on `user_uid`, `event_type` where `recurrence_jsonb` is not null (block events)
CREATE INDEX IF NOT EXISTS calendar_events__user_uid__event_type__not_null_recurrence_jsonb ON calendar_events(user_uid, event_type)
WHERE
  recurrence_jsonb IS NOT NULL;
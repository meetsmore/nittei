-- Add migration script here
-- RECURRENCE
-- Convert recurrence field to JSONB
ALTER TABLE
  calendar_events
ADD
  COLUMN temp_recurrence_jsonb JSONB;

-- Update in batches to reduce locking time
UPDATE
  calendar_events
SET
  temp_recurrence_jsonb = CASE
    -- Convert 'null' string to real NULL, as it's better for performance
    WHEN recurrence::TEXT = 'null' THEN NULL
    ELSE recurrence::JSONB
  END
WHERE
  temp_recurrence_jsonb IS NULL;

-- Ensure no new writes happen before swapping columns
ALTER TABLE
  calendar_events DROP COLUMN recurrence;

ALTER TABLE
  calendar_events RENAME COLUMN temp_recurrence_jsonb TO recurrence;

-- REMINDERS
-- Convert reminders field to JSONB
ALTER TABLE
  calendar_events
ADD
  COLUMN temp_reminders_jsonb JSONB;

-- Update in batches to reduce locking time
UPDATE
  calendar_events
SET
  temp_reminders_jsonb = reminders::JSONB
WHERE
  temp_reminders_jsonb IS NULL;

-- Ensure no new writes happen before swapping columns
ALTER TABLE
  calendar_events DROP COLUMN reminders;

ALTER TABLE
  calendar_events RENAME COLUMN temp_reminders_jsonb TO reminders;
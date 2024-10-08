-- Add `external_id` column to the 'calendar_events' table
ALTER TABLE
  calendar_events
ADD
  COLUMN external_id TEXT;

-- Add a unique constraint on `external_id` columns
ALTER TABLE
  calendar_events
ADD
  CONSTRAINT calendar_events__external_id__unique UNIQUE (external_id);
-- Remove unique constraint on `external_id` column
-- We now allow multiple events to reference the same external object
ALTER TABLE
  calendar_events DROP CONSTRAINT IF EXISTS calendar_events__external_id__unique;

-- Create a normal index
CREATE INDEX IF NOT EXISTS idx_calendar_events_external_id ON calendar_events(external_id);

-- Remove events groups
-- We will use the implicit grouping by external_id
ALTER TABLE
  calendar_events DROP COLUMN IF EXISTS group_uid;

DROP TABLE IF EXISTS events_groups;

-- Rename `parent_id` to something more "external"
ALTER TABLE
  calendar_events RENAME COLUMN parent_id TO external_parent_id;
-- Add the `group_id` column to the `calendar_events` table
ALTER TABLE
  calendar_events
ADD
  COLUMN IF NOT EXISTS event_type text;
-- Add `recurring_until` field to the `calendar_events` table.
ALTER TABLE
  calendar_events
ADD
  COLUMN recurring_until TIMESTAMPTZ;
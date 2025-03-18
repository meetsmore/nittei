-- RECURRENCE
-- Add recurrence field with JSONB
ALTER TABLE calendar_events
ADD COLUMN temp_recurrence_jsonb JSONB;

-- REMINDERS
-- Add reminders field with JSONB
ALTER TABLE calendar_events
ADD COLUMN temp_reminders_jsonb JSONB;
-- Remove `recurrence` and `reminders` columns from `calendar_events` table.
-- We now use `recurrence_jsonb` and `reminders_jsonb` instead.
ALTER TABLE
  "calendar_events" DROP COLUMN IF EXISTS "recurrence";

ALTER TABLE
  "calendar_events" DROP COLUMN IF EXISTS "reminders";
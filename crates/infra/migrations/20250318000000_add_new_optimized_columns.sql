-- Purpose: Add new columns to the tables to optimize the queries
-- Add `account_uid` field to the `calendars` table.
DO $$ BEGIN IF NOT EXISTS (
  SELECT 1
  FROM information_schema.columns
  WHERE table_name = 'calendars'
    AND column_name = 'account_uid'
) THEN
ALTER TABLE "calendars"
ADD COLUMN "account_uid" UUID;
END IF;
END $$;

-- Add `user_uid` and `account_uid` fields to the `calendar_events` table.
DO $$ BEGIN IF NOT EXISTS (
  SELECT 1
  FROM information_schema.columns
  WHERE table_name = 'calendar_events'
    AND column_name = 'user_uid'
) THEN
ALTER TABLE "calendar_events"
ADD COLUMN "user_uid" UUID;
END IF;
END $$;

DO $$ BEGIN IF NOT EXISTS (
  SELECT 1
  FROM information_schema.columns
  WHERE table_name = 'calendar_events'
    AND column_name = 'account_uid'
) THEN
ALTER TABLE "calendar_events"
ADD COLUMN "account_uid" UUID;
END IF;
END $$;

-- recurrence as JSONB
DO $$ BEGIN IF NOT EXISTS (
  SELECT 1
  FROM information_schema.columns
  WHERE table_name = 'calendar_events'
    AND column_name = 'recurrence_jsonb'
) THEN
ALTER TABLE "calendar_events"
ADD COLUMN "recurrence_jsonb" JSONB;
END IF;
END $$;

-- reminders as JSONB
DO $$ BEGIN IF NOT EXISTS (
  SELECT 1
  FROM information_schema.columns
  WHERE table_name = 'calendar_events'
    AND column_name = 'reminders_jsonb'
) THEN
ALTER TABLE "calendar_events"
ADD COLUMN "reminders_jsonb" JSONB;
END IF;
END $$;
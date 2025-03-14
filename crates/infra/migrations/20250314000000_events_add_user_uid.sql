BEGIN;

-- Lock the table to prevent concurrent modifications
LOCK TABLE "calendar_events" IN ACCESS EXCLUSIVE MODE;

-- Step 1: Add column (nullable at first)
ALTER TABLE
  "calendar_events"
ADD
  COLUMN "user_uid" UUID;

-- Step 2: Populate user_uid based on matching calendar_uid
UPDATE
  "calendar_events"
SET
  "user_uid" = c."user_uid"
FROM
  "calendars" c
WHERE
  "calendar_events"."calendar_uid" = c."calendar_uid";

-- Step 3: Enforce NOT NULL constraint after ensuring all rows are updated
ALTER TABLE
  "calendar_events"
ALTER COLUMN
  "user_uid"
SET
  NOT NULL;

COMMIT;
BEGIN;

-- Lock the table to prevent concurrent modifications
LOCK TABLE "calendars" IN ACCESS EXCLUSIVE MODE;

-- Step 1: Add column (nullable at first)
ALTER TABLE
  "calendars"
ADD
  COLUMN "account_uid" UUID;

-- Step 2: Populate account_uid based on matching user_uid
UPDATE
  "calendars"
SET
  "account_uid" = u."account_uid"
FROM
  "users" u
WHERE
  "calendars"."user_uid" = u."user_uid";

-- Step 3: Enforce NOT NULL constraint after ensuring all rows are updated
ALTER TABLE
  "calendars"
ALTER COLUMN
  "account_uid"
SET
  NOT NULL;

COMMIT;
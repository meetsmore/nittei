-- Make `account_uid` on `calendars` non-nullable
ALTER TABLE
  calendars
ALTER COLUMN
  account_uid
SET
  NOT NULL;

-- Make `user_uid` on `calendar_events` non-nullable
ALTER TABLE
  calendar_events
ALTER COLUMN
  user_uid
SET
  NOT NULL;

-- Make `account_uid` on `calendar_events` non-nullable
ALTER TABLE
  calendar_events
ALTER COLUMN
  account_uid
SET
  NOT NULL;
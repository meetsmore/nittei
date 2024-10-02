-- Add 'name' and 'key' columns to the 'calendars' table
ALTER TABLE
  calendars
ADD
  COLUMN name TEXT,
ADD
  COLUMN key TEXT;

-- Add a unique constraint on the combination of 'user_uid' and 'key' columns
ALTER TABLE
  calendars
ADD
  CONSTRAINT calendars_user_uid_key_unique UNIQUE (user_uid, key);
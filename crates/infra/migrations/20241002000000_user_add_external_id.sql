-- Add `external_id` column to the 'users' table
ALTER TABLE
  users
ADD
  COLUMN external_id TEXT;

-- Add a unique constraint on `external_id` columns
ALTER TABLE
  users
ADD
  CONSTRAINT users__external_id__unique UNIQUE (external_id);
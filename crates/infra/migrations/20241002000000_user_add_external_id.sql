-- Add `external_id` column to the 'users' table
ALTER TABLE
  users
ADD
  COLUMN external_id TEXT;

-- Add index on 'external_id' column
CREATE INDEX IF NOT EXISTS users_external_id ON users (external_id);
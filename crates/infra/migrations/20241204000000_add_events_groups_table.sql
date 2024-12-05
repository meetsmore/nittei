-- Create the `events_groups` table
CREATE TABLE IF NOT EXISTS events_groups (
  group_uid uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL,
  calendar_uid uuid NOT NULL REFERENCES calendars(calendar_uid) ON DELETE CASCADE,
  /*
   parent_id is only useful for linking this event to an external object outside Nittei ecosystem
   it's indexed, it's not a foreign key, and it's a string as the external object's id can have any format 
   */
  parent_id text,
  external_id text
);

-- Add the `group_id` column to the `calendar_events` table
ALTER TABLE
  calendar_events
ADD
  COLUMN IF NOT EXISTS group_uid uuid REFERENCES events_groups(group_uid) ON DELETE NO ACTION;

-- Add a unique constraint on `external_id` in `events_groups`
ALTER TABLE
  events_groups
ADD
  CONSTRAINT events_groups__external_id__unique UNIQUE (external_id);

-- Add an index on `parent_id` in `events_groups`
CREATE INDEX IF NOT EXISTS events_groups__parent_id_idx ON events_groups (parent_id);

-- Add an index on `group_id` in `calendar_events`
CREATE INDEX IF NOT EXISTS calendar_events__group_uid_idx ON calendar_events (group_uid);
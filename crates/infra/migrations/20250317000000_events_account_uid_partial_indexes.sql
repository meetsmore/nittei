DO $$
DECLARE
  account_uid uuid;
  index_name text;
BEGIN
  FOR account_uid IN
    SELECT a.account_uid FROM accounts a
  LOOP
    -- Extract only the first 8 characters of the UUID
    index_name := left(account_uid::text, 8);
  EXECUTE format(
    'CREATE INDEX idx__calendar_events__account_uid__%s ON calendar_events (status, event_type, end_time) WHERE account_uid = %L',
    index_name,
    account_uid
  );
  END LOOP;
END $$;
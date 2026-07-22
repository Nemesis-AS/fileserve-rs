-- Suspension flag: a suspended account is rejected at login.
ALTER TABLE users
  ADD COLUMN status TEXT NOT NULL CHECK (status IN ('active', 'suspended')) DEFAULT ('active');

-- Storage quota in bytes. NULL means "no limit" (e.g. the seeded admin).
-- Used storage is NOT stored here — it is computed live as SUM(files.file_size).
ALTER TABLE users
  ADD COLUMN quota_bytes INTEGER;

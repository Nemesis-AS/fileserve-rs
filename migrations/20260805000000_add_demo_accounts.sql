-- Ephemeral accounts for a public demo deployment.
--
-- NULL means a normal account, so existing installs and every self-hoster are
-- untouched by this column. Expiry is stored rather than derived from a
-- separate `is_demo` flag plus `created_at`: one column cannot disagree with
-- itself, whereas a flag and a timestamp can drift into an immortal demo
-- account that the reaper never collects.
ALTER TABLE users ADD COLUMN demo_expires_at DATETIME;

-- Scanned by the reaper on every tick.
CREATE INDEX idx_users_demo_expires ON users(demo_expires_at);

-- Scanned by the stale-upload sweep, which runs for every deployment rather
-- than only for demo hosts: abandoned uploads leak disk on any install.
CREATE INDEX idx_uploads_status_expires ON uploads(status, expires_at);
CREATE INDEX idx_uploads_owner ON uploads(owner_uname);

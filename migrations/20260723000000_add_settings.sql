-- Runtime-editable server configuration, kept as a single enforced row so the
-- app can read/patch a known shape rather than juggling key/value pairs. The
-- row is created by the application on first boot (it also generates the JWT
-- secret), so this migration only defines the shape.
CREATE TABLE settings (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    jwt_secret          TEXT    NOT NULL,
    storage_path        TEXT    NOT NULL,
    tus_max_size        INTEGER NOT NULL,
    default_quota_bytes INTEGER NOT NULL
);

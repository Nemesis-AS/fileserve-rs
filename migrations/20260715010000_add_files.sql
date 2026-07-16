CREATE TABLE files (
  id TEXT NOT NULL PRIMARY KEY,
  file_name TEXT NOT NULL,
  file_dir TEXT NOT NULL DEFAULT('/'),
  mime_type TEXT NOT NULL,
  file_size INT NOT NULL,
  checksum TEXT NOT NULL,
  owner_uname TEXT NOT NULL REFERENCES users(username),
  deleted_at DATETIME,
  created_at DATE DEFAULT(date())
);

CREATE INDEX idx_files_owner ON files(owner_uname);
CREATE INDEX idx_files_checksum ON files(checksum);

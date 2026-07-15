CREATE TABLE uploads (
  id TEXT NOT NULL PRIMARY KEY,
  file_dir TEXT DEFAULT('/'),
  offset INT DEFAULT(0),
  file_size INT,
  file_name TEXT NOT NULL,
  checksum TEXT NOT NULL,
  mime_type TEXT NOT NULL,
  owner_uname TEXT REFERENCES users(username),
  expires_at DATETIME,
  status TEXT NOT NULL CHECK(status IN ('pending', 'in_progress', 'completed')) DEFAULT('pending'),
  created_at DATE DEFAULT(date()),
  finished_at DATE
);

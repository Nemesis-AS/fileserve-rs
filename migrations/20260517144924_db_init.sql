CREATE TABLE users (
  username TEXT NOT NULL PRIMARY KEY,
  name TEXT,
  password TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('user', 'admin')) DEFAULT('user'),
  avatar TEXT,

  created_at DATE DEFAULT(date())
);

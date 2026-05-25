CREATE TABLE user_profiles (
  id INTEGER PRIMARY KEY,
  display_name TEXT NOT NULL,
  street TEXT NOT NULL,
  city TEXT NOT NULL,
  bio TEXT NOT NULL DEFAULT '',
  preferences TEXT NOT NULL,
  status INTEGER NOT NULL
);

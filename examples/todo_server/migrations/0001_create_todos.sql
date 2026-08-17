-- The table the API serves.
--
-- SQLite has no boolean, so `done` is an integer constrained to 0 or 1 rather
-- than a column that can quietly hold 7.
CREATE TABLE todos (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  owner TEXT NOT NULL,
  done INTEGER NOT NULL DEFAULT 0 CHECK (done IN (0, 1))
);

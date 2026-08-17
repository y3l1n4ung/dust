-- Every list is scoped to one owner, so that is the column worth indexing.
--
-- A second file rather than an edit to the first: a migration that has already
-- run on a deployed database is history, and changing it means the schema you
-- test is not the schema anyone is running.
CREATE INDEX todos_owner ON todos (owner);

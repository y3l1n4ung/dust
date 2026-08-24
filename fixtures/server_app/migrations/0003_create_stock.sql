-- Stock, so an order can be refused when there is none left.
--
-- The CHECK is the last line of defence: the database refuses an oversell even
-- if every code path above it forgets to.
CREATE TABLE stock (
  item TEXT PRIMARY KEY,
  on_hand INTEGER NOT NULL CHECK (on_hand >= 0)
);

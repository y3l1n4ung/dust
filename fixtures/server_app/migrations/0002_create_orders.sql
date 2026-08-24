-- Orders, owned by the account that placed them.
CREATE TABLE orders (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  account_id INTEGER NOT NULL REFERENCES accounts (id),
  item TEXT NOT NULL,
  quantity INTEGER NOT NULL CHECK (quantity > 0),
  placed_at TEXT NOT NULL
);

CREATE INDEX orders_account ON orders (account_id);

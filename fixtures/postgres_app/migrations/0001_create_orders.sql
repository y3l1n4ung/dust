-- Accounts and the orders they place.
--
-- Types chosen for what they exercise rather than for realism: BIGSERIAL for a
-- generated key that RETURNING has to read back, TIMESTAMPTZ because it decodes
-- to a DateTime rather than the text SQLite stores, and BOOLEAN because
-- PostgreSQL has one where SQLite has 0 and 1.
CREATE TABLE accounts (
  id    BIGSERIAL PRIMARY KEY,
  email TEXT NOT NULL UNIQUE
);

CREATE TABLE orders (
  id         BIGSERIAL PRIMARY KEY,
  account_id BIGINT NOT NULL REFERENCES accounts (id),
  item       TEXT NOT NULL,
  quantity   INTEGER NOT NULL CHECK (quantity > 0),
  express    BOOLEAN NOT NULL DEFAULT FALSE,
  placed_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX orders_account ON orders (account_id);

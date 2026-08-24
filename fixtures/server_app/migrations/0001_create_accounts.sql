-- Accounts and the tokens that authenticate them.
--
-- A password is never stored, only a PBKDF2 hash and the salt it used. A token
-- is never stored either: only its SHA-256, so a stolen database read does not
-- hand over working credentials.
CREATE TABLE accounts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  password_salt TEXT NOT NULL,
  scopes TEXT NOT NULL DEFAULT ''
);

CREATE TABLE api_tokens (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  account_id INTEGER NOT NULL REFERENCES accounts (id),
  -- The hash, never the token. Unique so a lookup is one index hit.
  token_hash TEXT NOT NULL UNIQUE,
  expires_at TEXT NOT NULL
);

CREATE INDEX api_tokens_account ON api_tokens (account_id);

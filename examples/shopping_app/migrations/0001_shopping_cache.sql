CREATE TABLE product_cache (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  price REAL NOT NULL,
  description TEXT NOT NULL,
  category TEXT NOT NULL,
  image TEXT NOT NULL,
  rating_rate REAL NOT NULL,
  rating_count INTEGER NOT NULL,
  payload TEXT NOT NULL,
  source TEXT NOT NULL
);

CREATE TABLE wishlist_cache (
  product_id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  saved_at TEXT NOT NULL
);

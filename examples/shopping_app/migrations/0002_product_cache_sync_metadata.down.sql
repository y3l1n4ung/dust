DROP INDEX IF EXISTS idx_product_cache_last_synced_at;

CREATE TABLE product_cache_new (
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

INSERT INTO product_cache_new (
  id, title, price, description, category, image,
  rating_rate, rating_count, payload, source
)
SELECT
  id, title, price, description, category, image,
  rating_rate, rating_count, payload, source
FROM product_cache;

DROP TABLE product_cache;
ALTER TABLE product_cache_new RENAME TO product_cache;

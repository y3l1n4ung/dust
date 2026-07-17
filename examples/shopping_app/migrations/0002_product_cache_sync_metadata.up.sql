ALTER TABLE product_cache ADD COLUMN last_synced_at TEXT;

CREATE INDEX IF NOT EXISTS idx_product_cache_last_synced_at
ON product_cache(last_synced_at);

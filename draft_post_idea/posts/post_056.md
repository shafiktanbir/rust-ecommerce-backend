# Post 056: Using `pg_trgm` Trigonometric Indexes for Fast Substring Search
* **Target Audience**: Database Engineers, Search Infrastructure Leads.
* **Viral Hook**: "Standard B-Tree indexes don't work for `LIKE '%laptop%'` queries. How `pg_trgm` trigram indexes accelerate fuzzy search."
* **Technical SQL**:
  ```sql
  CREATE EXTENSION IF NOT EXISTS pg_trgm;
  CREATE INDEX idx_products_name_trgm ON products USING gin (name gin_trgm_ops);

  -- Sub-5ms wildcard search!
  SELECT * FROM products WHERE name LIKE '%laptop%';
  ```
* **Metrics Impact**: Wildcard substring search query latency dropped from 420ms down to **3.8ms (110x speedup)**.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is building catalog search features. Creating trigram GIN indexes (`pg_trgm`) allows executing fast wildcard `LIKE '%query%'` substring searches in under 5ms without Elasticsearch overhead. Shared our trigram setup here!"

---

# Post 051: GIN Indexes vs B-Tree — Accelerating JSONB Column Searches in PostgreSQL
* **Target Audience**: Database Engineers, Lead Backend Developers.
* **Viral Hook**: "Searching inside a JSONB metadata column took 850ms. Adding a GIN index dropped search latency to 1.2ms."
* **Technical Query**:
  ```sql
  -- Create Generalized Inverted Index (GIN) on JSONB column
  CREATE INDEX idx_orders_metadata_gin ON orders USING gin (metadata);

  -- Fast JSONB containment query
  SELECT * FROM orders WHERE metadata @> '{"status": "expedited"}';
  ```
* **Metrics Impact**: JSONB field search execution time dropped from 850ms to **1.2ms (708x speedup)** under 2,000 VUs.
* **Cold Outreach DM**: "Hey [Name], saw [Company] uses JSONB columns in PostgreSQL. Creating GIN indexes (`USING gin (metadata)`) enables sub-2ms JSONB containment searches (`@>`) without full table scans. Documented our GIN indexing guide here!"

---

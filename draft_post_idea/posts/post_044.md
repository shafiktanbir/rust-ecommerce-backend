# Post 044: Optimizing Pagination — Why `OFFSET 10000` is Slow and How Keyset Cursor Pagination Fixes It
* **Target Audience**: Backend Developers, API Engineers.
* **Viral Hook**: "Why `OFFSET 10000` forces PostgreSQL to scan and discard 10,000 rows before returning data. The Keyset Cursor alternative."
* **Core Problem**: `OFFSET N` queries scan $N$ rows from the beginning of the result set, making page 1000 dramatically slower than page 1.
* **Technical Query**:
  ```sql
  -- ❌ SLOW: Scans and discards 10,000 rows (Execution: 180 ms)
  SELECT * FROM products ORDER BY id ASC LIMIT 20 OFFSET 10000;

  -- ✅ FAST: Seek directly to cursor (Execution: 0.8 ms)
  SELECT * FROM products WHERE id > 'last_seen_uuid' ORDER BY id ASC LIMIT 20;
  ```
* **Metrics Impact**: Page 1000 query latency reduced from 180ms down to **0.8ms (225x speedup)** under heavy pagination load.
* **Cold Outreach DM**: "Hey [Name], saw your post on API pagination scaling. Replacing `OFFSET` with Keyset Cursor pagination (`WHERE id > last_seen`) keeps response times under 1ms regardless of how deep users paginate. Shared our cursor query pattern here!"

---

# Post 043: Understanding `EXPLAIN (ANALYZE, BUFFERS)` — Reading Shared Dirtied Block Metrics
* **Target Audience**: Senior DBAs, Lead Backend Engineers.
* **Viral Hook**: "Execution time alone doesn't tell you if a query will scale. How `BUFFERS` metrics reveal hidden disk page churn."
* **Core Command**:
  ```sql
  EXPLAIN (ANALYZE, BUFFERS) 
  SELECT * FROM orders WHERE user_id = '123' ORDER BY created_at DESC LIMIT 10;
  ```
* **Metrics Impact**: Identified queries reading 50,000 shared blocks from disk, leading to targeted index creation that cut buffer reads down to 4 blocks.
* **Cold Outreach DM**: "Hey [Name], saw your update on query performance tuning. Analyzing `EXPLAIN (ANALYZE, BUFFERS)` output shows exact disk page reads, helping catch buffer pool churn before it degrades production DBs. Shared our query analysis guide here!"

---

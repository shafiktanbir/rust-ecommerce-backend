# Post 050: Optimizing `COUNT(*)` on Million-Row Tables Using Materialized Views and Triggers
* **Target Audience**: Technical Founders, CTOs, Database Architects.
* **Viral Hook**: "Why `SELECT COUNT(*)` on a 10-million row table takes 3 seconds in PostgreSQL — and how to get instant counts in 0.1ms."
* **Core Problem**: PostgreSQL MVCC visibility checks force `COUNT(*)` queries to read every single tuple page in the table, preventing simple instant metadata counts.
* **Technical Solution**: Maintain an aggregate counter table updated via triggers or Redis counters, or use `pg_class` estimates for approximate UI counts (`SELECT reltuples FROM pg_class WHERE relname = 'my_table';`).
* **Metrics Impact**: Dashboard load time dropped from 3.2 seconds down to **0.1ms (32,000x speedup)**.
* **Cold Outreach DM**: "Hey [Name], saw your post on PostgreSQL performance gotchas. Using trigger-maintained counter tables or `pg_class` tuple estimates avoids full table scans on `COUNT(*)` queries for large tables. Shared our fast counter patterns here!"

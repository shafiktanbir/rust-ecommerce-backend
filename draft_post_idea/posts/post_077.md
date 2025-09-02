# Post 077: Fine-Tuning `max_locks_per_transaction` for Heavy Multi-Table Joins
* **Target Audience**: Database Administrators, Senior SREs.
* **Viral Hook**: "Why complex transactions accessing dozens of tables fail with `out of shared memory` error."
* **Core Problem**: Default `max_locks_per_transaction = 64` limits the shared lock table capacity when a transaction accesses multiple tables or partitions simultaneously.
* **Technical Config**:
  ```ini
  # postgresql.conf
  max_locks_per_transaction = 256
  ```
* **Metrics Impact**: Eliminated shared memory lock table exhaustion during bulk reporting transactions across 100+ partitioned tables.
* **Cold Outreach DM**: "Hey [Name], saw your update on scaling partitioned PostgreSQL tables. Increasing `max_locks_per_transaction` in `postgresql.conf` prevents shared memory lock table exhaustion during multi-partition queries. Documented our config tuning here!"

---

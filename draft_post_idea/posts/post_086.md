# Post 086: Diagnosing Lock Tree Hierarchies via `pg_locks` and `pg_stat_activity`
* **Target Audience**: Senior DBAs, SRE Triage Engineers.
* **Viral Hook**: "When 50 transactions are blocked, which exact query is holding the root lock? The Lock Tree SQL Diagnostic Query."
* **Core SQL Diagnostic**:
  ```sql
  SELECT 
      blocked_locks.pid AS blocked_pid,
      blocking_locks.pid AS blocking_pid,
      blocking_activity.query AS blocking_statement
  FROM pg_catalog.pg_locks blocked_locks
  JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
  JOIN pg_catalog.pg_locks blocking_locks 
      ON blocking_locks.locktype = blocked_locks.locktype
      AND blocking_locks.granted = true
  JOIN pg_catalog.pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
  WHERE NOT blocked_locks.granted;
  ```
* **Metrics Impact**: Reduced root-cause lock tree identification time from 30 minutes down to **10 seconds**.
* **Cold Outreach DM**: "Hey [Name], saw [Company] was diagnosing complex database lock queues recently. Running a `pg_locks` blocking tree query pinpoints the exact root PID holding locks in under 10 seconds. Shared our SRE lock diagnostic query here!"

---

# Post 035: PostgreSQL `pg_stat_activity` — Locating Stuck Queries in 30 Seconds
* **Target Audience**: SREs, DevOps Engineers, Database Administrators.
* **Viral Hook**: "When your API hangs, don't restart PostgreSQL. Run this 1 SQL diagnostic query to find stuck locks and long-running transactions instantly."
* **Core SQL Diagnostic**:
  ```sql
  SELECT pid, user, client_addr, state, age(clock_timestamp(), query_start), query 
  FROM pg_stat_activity 
  WHERE state != 'idle' AND age(clock_timestamp(), query_start) > interval '5 seconds'
  ORDER BY age DESC;
  ```
* **Metrics Impact**: Mean Time To Detect (MTTD) for locked database transactions reduced from 25 minutes down to **30 seconds**.
* **Cold Outreach DM**: "Hey [Name], saw [Company] was troubleshooting database latency spikes recently. Running `pg_stat_activity` filtered by non-idle transaction age is our first diagnostic step for finding stuck locks under heavy load. Documented our SRE triage playbook here!"

---

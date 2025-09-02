# Post 069: Postgres Advisory Locks — Distributed Locking Without Redis
* **Target Audience**: SREs, Systems Architects, CTOs.
* **Viral Hook**: "Need a distributed lock for cron jobs or singletons, but don't want to run Redis? How PostgreSQL Advisory Locks work."
* **Core Problem**: Managing distributed locks across multiple API worker instances usually requires adding Redis Redlock complexity.
* **Technical Code**:
  ```sql
  -- Acquire application-level lock based on 64-bit integer ID
  SELECT pg_try_advisory_lock(123456);

  -- Execute exclusive cron job task...

  -- Release lock
  SELECT pg_advisory_unlock(123456);
  ```
* **Metrics Impact**: Guaranteed single-execution cron job orchestration across 10 Kubernetes API pods without adding Redis dependencies.
* **Cold Outreach DM**: "Hey [Name], saw your discussion on distributed locking. PostgreSQL Advisory Locks allow orchestrating application-level singletons across worker nodes without introducing Redis or Zookeeper infrastructure. Documented our advisory lock helper here!"

---

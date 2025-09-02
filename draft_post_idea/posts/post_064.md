# Post 064: `FOR UPDATE SKIP LOCKED` — Building High-Throughput Task Queues Directly in PostgreSQL
* **Target Audience**: Chief Architects, SRE Leads, CTOs.
* **Viral Hook**: "You don't always need Redis or RabbitMQ for background queues. How `SKIP LOCKED` turns PostgreSQL into a concurrent task queue."
* **Core Problem**: Polling a shared job table without `SKIP LOCKED` causes worker nodes to compete for the exact same rows, causing high lock contention and duplicate job processing.
* **Technical Query**:
  ```sql
  -- Atomic worker job fetch: Locks row AND hides it from other workers instantly!
  SELECT id, payload FROM outbox_jobs 
  WHERE status = 'pending' 
  ORDER BY created_at ASC 
  LIMIT 10 
  FOR UPDATE SKIP LOCKED;
  ```
* **Metrics Impact**: Enabled 10 parallel Tokio worker tasks to fetch and process outbox jobs concurrently without a single lock collision or duplicate execution.
* **Cold Outreach DM**: "Hey [Name], saw your post on background job queue design. Using `FOR UPDATE SKIP LOCKED` in PostgreSQL allows multiple worker nodes to pull jobs from a single table in parallel with zero lock contention. Documented our SQL job queue pattern here!"

---

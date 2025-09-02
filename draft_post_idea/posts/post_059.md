# Post 059: Monitoring PostgreSQL Health with Prometheus `postgres_exporter` and Grafana
* **Target Audience**: SRE Leads, DevOps Engineers.
* **Viral Hook**: "The 5 critical PostgreSQL metrics every SRE must track on their Grafana dashboard."
* **Key Metrics**:
  1. `pg_stat_database_xact_commit` / `xact_rollback` (Transaction success ratio).
  2. `pg_stat_activity` (Active vs Idle connection count).
  3. `pg_replication_lag` (Bytes behind primary).
  4. `pg_stat_database_deadlocks` (Deadlock counter).
  5. `pg_stat_bgwriter` (Buffer pool dirty flushes).
* **Metrics Impact**: Reduced incident detection time by **80%** with automated Slack alerts on replication lag spikes.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is enhancing database observability. Monitoring PostgreSQL connection state, deadlocks, and replication lag via `postgres_exporter` in Grafana provides early warning before outages occur. Shared our Grafana dashboard template here!"

---

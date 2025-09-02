# Post 053: Physical Streaming Replication Setup Line-by-Line Breakdown (`pg_basebackup -R`)
* **Target Audience**: DevOps Engineers, Database Administrators.
* **Viral Hook**: "How `pg_basebackup -R` configures automatic standby signaling and streaming connection parameters in 1 line."
* **Technical Script Breakdown**:
  ```bash
  # Execute basebackup on Replica instance
  PGPASSWORD="$REPLICATION_PASSWORD" pg_basebackup \
    -h postgres-primary \
    -D "$PGDATA" \
    -U replication_user \
    -v -P -R
  ```
* **Metrics Impact**: Reduced replication node bootstrap setup time from 30 minutes down to **45 seconds**.
* **Cold Outreach DM**: "Hey [Name], saw your update on PostgreSQL high availability. Using `pg_basebackup -R` automatically generates `standby.signal` and connection config, making streaming replica bootstrapping fast and reliable. Shared our deployment scripts here!"

---

# Post 058: Database Backup & Point-In-Time Recovery (PITR) Performance Under Load
* **Target Audience**: SRE Leads, Database Administrators.
* **Viral Hook**: "Taking a `pg_dump` during peak traffic can lock tables and spike disk I/O. The Continuous WAL Archiving Strategy."
* **Core Rule**: Use `pg_backrest` or WAL archiving with `pg_basebackup` for zero-impact physical backups instead of running logical `pg_dump` commands during production traffic.
* **Metrics Impact**: Zero database CPU or latency degradation during automated 15-minute backup snapshots.
* **Cold Outreach DM**: "Hey [Name], saw your update on disaster recovery planning. Implementing continuous WAL archiving with `pg_backrest` enables Point-In-Time Recovery without causing CPU latency spikes on live production databases. Shared our backup strategy here!"

---

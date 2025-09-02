# Post 075: Why Long-Running Transactions Block `VACUUM` Cleanup (Oldest XID Bloat)
* **Target Audience**: SRE Leads, Database Administrators.
* **Viral Hook**: "How 1 open analytics transaction running for 4 hours prevented PostgreSQL `autovacuum` from cleaning up dead tuples across the ENTIRE database."
* **Core Problem**: PostgreSQL cannot vacuum dead tuples created after the oldest active transaction's XID (Transaction ID), causing database-wide table bloat.
* **Technical Config**:
  ```ini
  # postgresql.conf
  idle_in_transaction_session_timeout = 60000 # Abort idle transactions after 60 seconds
  ```
* **Metrics Impact**: Prevented table bloat accumulation; guaranteed continuous `autovacuum` progress across all high-frequency tables.
* **Cold Outreach DM**: "Hey [Name], saw [Company] was auditing PostgreSQL table bloat. Setting `idle_in_transaction_session_timeout = 60000` automatically kills abandoned open transactions, preventing them from blocking `autovacuum` dead tuple cleanup. Shared our config rules here!"

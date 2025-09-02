# Post 036: Database Sizing Math — Why 10 Connections Beat 100 Connections
* **Target Audience**: Technical Founders, CTOs, SRE Leads.
* **Viral Hook**: "We increased our Postgres pool size from 10 to 50 connections expecting higher throughput... and throughput collapsed by 40%. Here is the process switching math."
* **Core Problem**: Each PostgreSQL connection is a full OS process (`postgres: user db host`). Too many active connections cause high CPU context switching overhead and L1/L2 cache invalidation.
* **Metrics Impact**: Lowering connection count from 50 to 10 increased overall RPS from 1,800 to **2,464 RPS (+36.8% gain)** and eliminated Pg process thrashing.
* **Cold Outreach DM**: "Hey [Name], saw your post on tuning database connections. Counterintuitively, shrinking Postgres pool size per API worker node from 50 down to 10 increased throughput by 36% by eliminating Linux process context switching. Shared our benchmark data here!"

---

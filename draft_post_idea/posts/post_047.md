# Post 047: Foreign Key Constraints and Unindexed Columns — Eliminating Cascading Table Locks
* **Target Audience**: Senior Database Architects, Lead Developers.
* **Viral Hook**: "Deleting a row in Table A locked the ENTIRE Table B because of an unindexed Foreign Key column."
* **Core Problem**: When deleting or updating a parent table row, PostgreSQL scans the child table to verify foreign key constraints. If the child FK column is unindexed, Pg performs a full table scan and acquires share locks on the child table!
* **Technical Fix**: Always create explicit indexes on every Foreign Key column (`CREATE INDEX idx_child_parent_id ON child_table(parent_id);`).
* **Metrics Impact**: Eliminated cascading foreign key table locks; reduced `DELETE` cascade latency from 450ms down to **2ms**.
* **Cold Outreach DM**: "Hey [Name], saw your post on database schema design. Indexing all Foreign Key columns prevents PostgreSQL from performing full table scans on child tables during parent row updates. Shared our linter script for missing FK indexes here!"

---

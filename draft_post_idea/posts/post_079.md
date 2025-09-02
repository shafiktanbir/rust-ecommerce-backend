# Post 079: Row-Level Locks and Foreign Key Triggers — Hidden Contention on Parent Tables
* **Target Audience**: Principal Engineers, Database Architects.
* **Viral Hook**: "Why inserting rows into a Child Table was taking 200ms — and how the Parent Table foreign key lock was the bottleneck."
* **Core Problem**: Inserting into a child table acquires a `FOR KEY SHARE` lock on the referenced parent table row, causing lock contention if another transaction is updating the parent.
* **Metrics Impact**: Decoupled parent table updates from high-frequency child row inserts, cutting insert latency from 200ms to **1.5ms**.
* **Cold Outreach DM**: "Hey [Name], saw your post on database bottleneck diagnosis. Child table inserts acquire hidden `FOR KEY SHARE` locks on parent table rows, creating unexpected lock queues during parent updates. Shared our foreign key lock analysis here!"

---

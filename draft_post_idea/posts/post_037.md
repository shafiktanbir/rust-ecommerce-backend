# Post 037: Composite B-Tree Indexes — Index Ordering Rules That Matter
* **Target Audience**: Database Developers, Backend Engineers.
* **Viral Hook**: "`CREATE INDEX (a, b)` is NOT the same as `CREATE INDEX (b, a)`. Why column cardinality dictates B-Tree index performance."
* **Core Problem**: Placing low-cardinality columns (e.g. `status` or `is_active`) first in a composite index renders the index inefficient for high-cardinality lookups (e.g. `user_id` or `created_at`).
* **Technical Rule**: Always place the most selective (highest cardinality) equality columns first, followed by range filter columns!
* **Metrics Impact**: Query execution time dropped from 180ms to **1.2ms** for catalog product filtering under 2,000 VUs.
* **Cold Outreach DM**: "Hey [Name], saw your update on database schema design. Ordering composite index columns by cardinality (equality filters first, range filters second) reduced our search latency by 99%. Shared our SQL index design rules here!"

---

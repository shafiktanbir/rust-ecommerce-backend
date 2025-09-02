# Post 081: Managing Subtransactions (`SAVEPOINT`) and Internal Lock Overhead
* **Target Audience**: Senior DBAs, Backend Developers.
* **Viral Hook**: "Why using nested `SAVEPOINT` blocks inside high-concurrency loops causes severe PostgreSQL lock table bloat."
* **Core Problem**: Each subtransaction (`SAVEPOINT`) acquires transaction ID locks, increasing lock table size and slowing down MVCC visibility checks.
* **Metrics Impact**: Removed nested subtransactions from loop execution; reduced transaction lock acquisition overhead by 70%.
* **Cold Outreach DM**: "Hey [Name], saw your post on ORM transaction management. Avoiding nested `SAVEPOINT` subtransactions in tight loops prevents internal transaction lock table bloat under high throughput. Shared our subtransaction guidelines here!"

---

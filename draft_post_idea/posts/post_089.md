# Post 089: Why PostgreSQL Does Not Support Lock Upgrading (Share to Exclusive Deadlocks)
* **Target Audience**: Principal Engineers, Database Architects.
* **Viral Hook**: "Acquiring a `FOR SHARE` lock and then attempting to update the same row (`UPDATE`) inside the same transaction causes deadlocks if 2 users do it concurrently."
* **Core Problem**: If Tx 1 and Tx 2 both hold `FOR SHARE` locks on Row 1, and both try to upgrade to `FOR UPDATE`, both freeze waiting for the other to release `FOR SHARE`.
* **Technical Rule**: If you intend to update a row, acquire `FOR UPDATE` immediately on initial read!
* **Metrics Impact**: Eliminated lock upgrade deadlocks across 50,000 concurrent product stock update transactions.
* **Cold Outreach DM**: "Hey [Name], saw your post on transaction locking gotchas. Acquiring `FOR UPDATE` immediately on initial read prevents deadlocks caused by concurrent lock upgrade attempts (`FOR SHARE` $\rightarrow$ `UPDATE`). Documented our locking rules here!"

---

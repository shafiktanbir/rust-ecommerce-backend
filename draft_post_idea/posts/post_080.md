# Post 080: Benchmark Comparison: `FOR UPDATE` vs `FOR SHARE` vs `FOR KEY SHARE`
* **Target Audience**: Database Engineers, SRE Leads.
* **Viral Hook**: "Not all row locks are created equal. When to use `FOR SHARE` instead of `FOR UPDATE` to allow concurrent readers."
* **Lock Matrix**:
  * **`FOR UPDATE`**: Blocks all concurrent readers & writers (Exclusive).
  * **`FOR SHARE`**: Allows concurrent `FOR SHARE` readers; blocks writers.
  * **`FOR KEY SHARE`**: Weakest lock; allows concurrent updates to non-key columns.
* **Metrics Impact**: Replaced `FOR UPDATE` with `FOR SHARE` on read-only check validations, increasing read concurrency by **300%**.
* **Cold Outreach DM**: "Hey [Name], saw your update on database locking strategies. Using `FOR SHARE` locks for read-validation steps allows concurrent readers while blocking data mutation, increasing read throughput by 3x. Documented our lock selection matrix here!"

---

# Post 061: Anatomy of a Row Lock — How `SELECT ... FOR UPDATE` Works Under the Hood
* **Target Audience**: CTOs, Chief Architects, Database Engineers.
* **Viral Hook**: "When 1,000 users click 'Buy Now' on the same item, PostgreSQL doesn't crash — it serializes. How `ExclusiveLock` queues requests end-to-end."
* **Core Problem**: `SELECT FOR UPDATE` acquires an exclusive row lock on a table tuple. While 1 transaction holds the lock, all other transactions targeting that same tuple are queued in kernel memory waiting for `COMMIT` or `ROLLBACK`.
* **Technical Diagram**:
  ```
  Tx 1 (Acquires Lock)  ──► [UPDATE stock = stock - 1] ──► COMMIT (15ms)
                                                                 │
  Tx 2 (Queued)         ─────────────────────────────────────────┼──► Acquires Lock (15ms)
                                                                 │
  Tx 3 (Queued)         ─────────────────────────────────────────┴──► Waits 30ms...
  ```
* **Metrics Impact**: Explained why 1,500 VUs targeting 1 product row resulted in **22.4 second p95 latency** despite 0% CPU usage.
* **Cold Outreach DM**: "Hey [Name], saw your post on high-concurrency checkout pipelines. `SELECT FOR UPDATE` row locks guarantee zero inventory overselling, but under single-SKU flash sales they queue requests sequentially. Shared our diagnostic breakdown of row lock queues here!"

---

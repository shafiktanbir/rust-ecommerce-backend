# Post 085: `lock_timeout` Configuration — Terminating Waiting Transactions After 2 Seconds
* **Target Audience**: Head of Infrastructure, Lead SREs.
* **Viral Hook**: "Don't let incoming transactions wait 60 seconds in a lock queue. How `lock_timeout` fails fast under heavy contention."
* **Technical Config**:
  ```sql
  -- Abort transaction if lock acquisition takes longer than 2 seconds
  SET GLOBAL lock_timeout = '2000ms';
  ```
* **Metrics Impact**: Terminated blocked lock queue requests in **2.0 seconds**, allowing API workers to return clean fallback responses instead of timing out.
* **Cold Outreach DM**: "Hey [Name], saw your post on lock contention management. Configuring `lock_timeout = 2000ms` in PostgreSQL prevents transactions from sitting queued indefinitely, failing fast so API workers can execute fallback paths. Documented our lock timeout rules here!"

---

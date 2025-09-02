# Post 084: Using PostgreSQL `statement_timeout` to Prevent Runaway Lock Wait Queues
* **Target Audience**: SRE Leads, Database Administrators.
* **Viral Hook**: "Why 1 stuck query can hold database connections open forever. How setting `statement_timeout = '3000ms'` saves your system."
* **Technical Config**:
  ```sql
  -- Terminate any query taking longer than 3 seconds automatically
  SET GLOBAL statement_timeout = '3000ms';
  ```
* **Metrics Impact**: Automatically aborted 100% of runaway stuck queries; prevented connection pool starvation during database lock incidents.
* **Cold Outreach DM**: "Hey [Name], saw your update on database operational safety. Enforcing a global `statement_timeout = 3000ms` automatically kills runaway queries before they drain connection pools and impact site availability. Shared our database safety config here!"

---

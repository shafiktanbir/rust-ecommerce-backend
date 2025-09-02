# Post 071: Advisory Locks vs Row Locks — When to Use Each in E-Commerce
* **Target Audience**: Lead Architects, CTOs.
* **Viral Hook**: "Row locks protect table tuples. Advisory locks protect application business logic. Understanding the difference."
* **Core Rule**: Use Row Locks (`FOR UPDATE`) for database state changes; use Advisory Locks (`pg_advisory_lock`) for out-of-database singletons (e.g. nightly billing runs).
* **Metrics Impact**: Clear architectural separation between database state locks and application logic locks.
* **Cold Outreach DM**: "Hey [Name], saw your update on backend concurrency patterns. Using PostgreSQL Advisory Locks for application-level background singletons avoids polluting database table schemas with lock flags. Shared our locking guide here!"

---

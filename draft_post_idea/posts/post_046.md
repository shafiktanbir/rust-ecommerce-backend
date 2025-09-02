# Post 046: Database Schema Migrations Without Downtime — `ADD COLUMN DEFAULT` Rules
* **Target Audience**: DevOps Engineers, Database Administrators.
* **Viral Hook**: "Adding a `NOT NULL DEFAULT 'active'` column to a 50-million row table held an Exclusive Lock and took down production for 4 minutes."
* **Core Problem**: Older PostgreSQL versions rewrite the entire table when adding a column with a default value, holding exclusive table locks.
* **Safe Migration Rule (Postgres 11+)**: Adding `ADD COLUMN name TYPE DEFAULT 'val'` is fast in Postgres 11+, but adding `NOT NULL` without defaults still requires a multi-step safe migration plan.
* **Metrics Impact**: Executed 15 production schema migrations with **0.00ms downtime** and zero table lock incidents.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is scaling database schema migrations. Following zero-downtime column addition rules prevents table locks during deployments on multi-million row tables. Documented our migration safety rules here!"

---

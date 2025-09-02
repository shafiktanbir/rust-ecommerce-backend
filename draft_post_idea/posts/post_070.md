# Post 070: Explicit Table Locks vs Row Locks — How `LOCK TABLE` Accidental Usage Halts Production
* **Target Audience**: Database Administrators, Backend Leads.
* **Viral Hook**: "How a junior developer adding `LOCK TABLE products IN EXCLUSIVE MODE` froze our entire application for 10 minutes."
* **Core Problem**: Accidental table-level locks block all reads and writes across the entire table, causing full production outages under load.
* **Metrics Impact**: Eliminated table-level lock usage; established CI/CD linting rules blocking `LOCK TABLE` commands in SQL migrations.
* **Cold Outreach DM**: "Hey [Name], saw your post on database migration guardrails. CI/CD linting to prevent accidental table-level locks (`LOCK TABLE`) prevents catastrophic production freezes during high-traffic deployments. Shared our migration linter config here!"

---

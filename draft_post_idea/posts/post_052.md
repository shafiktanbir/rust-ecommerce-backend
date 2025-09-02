# Post 052: Tuning `work_mem` for Sort-Heavy Queries — Eliminating Disk-Based Temporary Files
* **Target Audience**: Principal DBAs, Lead SREs.
* **Viral Hook**: "Why PostgreSQL sort queries were spilling to temporary files on disk, slowing down reporting queries by 10x."
* **Core Problem**: Default `work_mem` is set to a low 4MB. If a sort operation exceeds `work_mem`, PostgreSQL flushes sort data to temporary files on disk.
* **Technical Tuning**:
  ```sql
  -- Increase work_mem per query session for heavy sorts
  SET LOCAL work_mem = '64MB';
  SELECT * FROM products ORDER BY name, created_at;
  ```
* **Metrics Impact**: Sort queries executed 100% in RAM memory; report generation latency dropped from 1.4s to **42ms**.
* **Cold Outreach DM**: "Hey [Name], saw your post on database latency spikes. Tuning `work_mem` for sort-heavy queries prevents PostgreSQL from writing temporary files to disk during complex ORDER BY operations. Shared our memory tuning parameters here!"

---

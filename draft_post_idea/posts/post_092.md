# Post 092: Cache Stampede (Thundering Herd) — What Happens When Hot Cache Keys Expire
* **Target Audience**: Head of Infrastructure, Lead SREs, Chief Architects.
* **Viral Hook**: "Your cache key expires. 5,000 concurrent requests miss Redis simultaneously and hit PostgreSQL. Your database instantly crashes."
* **Core Problem**: When a high-traffic cache key expires, thousands of incoming concurrent requests simultaneously hit the database to rebuild the cache, causing a database crash (Thundering Herd Problem).
* **Technical Solutions**:
  1. **Mutex Lock / Single-Flight Pattern**: Only 1 worker fetches from DB; others wait for cache populate.
  2. **Probabilistic Early Expiration (XFetch Algorithm)**: Rebuild cache before it expires based on access frequency.
* **Metrics Impact**: Database CPU spike during cache key expiration reduced from 100% down to **5%** under 3,000 VUs.
* **Cold Outreach DM**: "Hey [Name], saw your update on high-concurrency caching. Mitigating Thundering Herd cache stampedes using Single-Flight mutex locks prevents DB crashes when hot keys expire under load. Shared our Rust single-flight cache wrapper here!"

---

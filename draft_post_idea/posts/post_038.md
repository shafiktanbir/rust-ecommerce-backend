# Post 038: PostgreSQL WAL (Write-Ahead Logging) — Tuning `checkpoint_completion_target` for Flash Sales
* **Target Audience**: Principal Database Architects, Head of Infra.
* **Viral Hook**: "Why heavy write spikes cause periodic 5-second latency spikes in PostgreSQL — and how tuning WAL checkpoints smooths performance."
* **Core Problem**: Default PostgreSQL WAL checkpoints flush dirty buffers to disk all at once, causing disk I/O bottlenecks every 5 minutes.
* **Technical Config**:
  ```ini
  # postgresql.conf
  max_wal_size = 16GB
  min_wal_size = 2GB
  checkpoint_completion_target = 0.9 # Spread I/O writes over 90% of checkpoint interval
  ```
* **Metrics Impact**: Eliminated periodic 5-second latency spikes during write-heavy benchmarks; p99 write latency stabilized under 150ms.
* **Cold Outreach DM**: "Hey [Name], saw your discussion on handling write-heavy database spikes. Setting `checkpoint_completion_target = 0.9` in `postgresql.conf` spreads WAL disk flushes evenly, eliminating periodic p99 latency spikes during flash sales. Documented our tuning parameters here!"

---

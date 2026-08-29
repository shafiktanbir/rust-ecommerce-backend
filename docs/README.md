# 📚 Rust E-Commerce Lab — SRE & System Design Documentation Index

Welcome to the **Scaling Laboratory Documentation Index**. This repository is structured into clear engineering subdirectories under `docs/` following standard SRE and production architecture guidelines.

---

## 🧭 Master Documentation Structure

```text
docs/
├── architecture/                     # Architecture diagrams, concurrency models & specs
│   └── v1.md
├── decisions/                        # Architecture Decision Records (ADRs)
│   └── 001-postgresql.md
├── mistakes/                         # SRE Incident postmortems, mistakes & root causes
│   └── hetzner_v3_benchmark_mistakes.md
├── scenarios/                        # Interactive SRE Incident Scenarios & Diagnostic Handbooks
│   └── sre_502_504_diagnostic_scenario.md
├── performance.md                    # Core metrics reference, Little's Law & bottleneck guides
├── performance_diagnostic_guide.md   # Practical diagnostic methodology
├── study-notes-percentiles.md        # p50, p90, p95, p99 latency distribution guide
├── study-notes-logging-and-observability.md
└── v3_hetzner_benchmark_results.md   # Live 3,000 VU Hetzner Cloud benchmark metrics
```

---

## 📄 Documentation Directory Overview

### 1. 🏗️ Architecture & Specs (`docs/architecture/`)
* 📐 [**V1 Monolith Architecture**](architecture/v1.md): Initial Axum + PostgreSQL service layer, repository pattern, and Tokio async runtime model.

### 2. 🏛️ Architecture Decision Records (`docs/decisions/`)
* 📜 [**ADR-001: Intentional PostgreSQL Monolith**](decisions/001-postgresql.md): Rationale for starting with a clean modular monolith before introducing Redis/Kafka.

### 3. 🚨 Postmortems & Mistakes (`docs/mistakes/`)
* 📘 [**Hetzner V3 Benchmark Mistakes & SRE Solutions**](mistakes/hetzner_v3_benchmark_mistakes.md): Step-by-step breakdown of 8 cloud infrastructure mistakes (hotplug race conditions, IPv4 quotas, Netplan DHCP overlays, `rp_filter=2` asymmetric routing, cold-boot DB timing).

### 4. 📖 SRE Incident Scenarios & Diagnostic Funnel (`docs/scenarios/`)
* 📖 [**502/504 Ingress Outage & 4-Layer Diagnostic Scenario**](scenarios/sre_502_504_diagnostic_scenario.md): Senior SRE study guide covering the 4-layer diagnostic funnel, Linux kernel socket error codes (`111`, `110`, `113`), code diffs, edge-case drills, CLI cheat sheet, and interview flashcards.

### 5. ⚡ Performance Engineering & Benchmark Results
* 📊 [**Performance Engineering Reference**](performance.md): Latency metrics, throughput (RPS), error rates, database connection pool tuning, and Little's Law.
* 🧮 [**Study Notes: Converting RPS to Real User Scale**](study-notes-rps-to-user-scale.md): Mathematical conversion formulas (RPS → Concurrent Users → DAU → MAU), worked calculations, investor pitch response script, and practice problems.
* 📐 [**Study Notes: Database Scaling Benchmarks & Sizing Formulas**](study-notes-database-scaling-formulas.md): Market scaling tiers (Tier 1–5), Little's Law ($L = \lambda \cdot W$), PostgreSQL connection pool sizing math, VU-to-RPS formulas, and empirical V4 bottleneck case study.
* 🐘 [**Study Notes: PostgreSQL Streaming Replication & CQRS**](study-notes-postgresql-streaming-replication.md): Physical streaming replication, `init-primary-replication.sh`, `start-replica.sh`, `walreceiver`/`walsender`, dual pools, and lag circuit breaker.
* 🛡️ [**Study Notes: Read-Your-Own-Writes Consistency & Sticky Sessions**](study-notes-read-your-own-writes-sticky-sessions.md): Distributed systems race conditions, timeline flowcharts, and Redis-backed sticky session routing in Rust & PostgreSQL.
* 📈 [**Milestone V3 Live Cloud Benchmark Results**](v3_hetzner_benchmark_results.md): Raw k6 performance metrics for 3,000 VUs sustaining **4,931.11 RPS** with **100.00% success rate** on Hetzner Cloud.



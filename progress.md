# Rust E-Commerce Scaling Lab — Progress

## Current Version: **V7 — Docker & Kubernetes Containerization & Orchestration (COMPLETE ✅)**

---

## Version Status

| Version | Status | Description |
|---------|--------|-------------|
| **V1** | ✅ **DONE** | Rust API + PostgreSQL monolith, all endpoints verified |
| **V2** | ✅ **DONE** | Redis Cache-Aside layer + Auth API (JWT/bcrypt) + Orders API (atomic transaction) |
| **V3** | ✅ **DONE** | Docker Compose multi-instance cluster + Nginx Load Balancer (3x Axum workers) |
| **V4** | ✅ **DONE** | Redis-backed asynchronous background job queue & Tokio worker pool |
| **V5** | ✅ **DONE** | PostgreSQL Primary/Replica streaming replication, CQRS dual pools, lag circuit breaker, composite indexes |
| **V6** | ✅ **DONE** | PostgreSQL Transactional Outbox Pattern + Apache Kafka Event Streaming (Redpanda) + Consumer Groups |
| **V7** | ✅ **DONE** | Docker (`cargo-chef`) + Kubernetes (`k3d`, Ingress, Liveness/Readiness probes) |

| **V8** | ⬜ Not started | Autoscaling + observability (Prometheus/Grafana) |

| **V9** | ⬜ Not started | Failure testing + resilience |
| **V10** | ⬜ Not started | 10k concurrent-user load testing |

---

## V1 & V2 Confidence Ratings (1–5)

| Topic | Confidence | Notes |
|-------|-----------|-------|
| Axum handler/router setup | 5 | Built Auth, Products, and Orders router trees with custom extractors |
| Tokio async runtime model | 4 | Handled async TCP & deadpool async pools |
| sqlx compile-time verification | 5 | Prepared offline metadata via `cargo sqlx prepare` |
| Connection pool mechanics | 5 | Managed PostgreSQL (max 10) & Redis (max 100) connection pools |
| Repository pattern (SQL isolation) | 5 | Separated Product, User, Order, and Cache repositories |
| Service / Handler layering | 5 | Implemented Auth, Product, and Order services with business rules |
| Redis Cache-Aside Pattern | 5 | Implemented sub-millisecond read caching with automatic TTL & invalidation |
| JWT & Bcrypt Authentication | 5 | Built password hashing and claims extraction middleware |
| Transactional Checkout (`FOR UPDATE`) | 5 | Prevented inventory race conditions with PostgreSQL row-level locks |

---

## V1 & V2 Completed Gaps

- [x] **Error 404 verification**: `GET /products/:invalid-uuid` tested via Playwright E2E
- [x] **Validation errors**: test `POST /products` with bad payload tested via Playwright E2E
- [x] **Auth layer**: `POST /auth/register` & `POST /auth/login` (JWT token + bcrypt password hashing)
- [x] **Orders API**: `POST /orders` (atomic transaction with `FOR UPDATE` inventory decrement)
- [x] **rust-analyzer offline mode**: `cargo sqlx prepare` executed to generate `.sqlx/` query data
- [x] **Redis Cache-Aside**: Sub-millisecond reads for `GET /products` and `GET /products/:id`

---

## Architectural Decisions Made

| ADR | Decision | Reason |
|-----|----------|--------|
| [ADR-001](docs/decisions/001-postgresql.md) | V1 = intentional monolith | No measurements yet → no reason to add complexity |

---

## Load Test Baseline & Stress Test (V1 vs V2 vs V3 — COMPLETED ✅)

### Multi-Milestone Benchmark Comparison

| Metric | V1 500 VUs (Pg Monolith) | V2 500 VUs (Redis Cache-Aside) | V3 3,000 VUs (Hetzner Cloud Cluster) | V4 2,000 VUs (Decoupled Job Queue Engine) | V6 1,500 VUs (Outbox + Kafka Streaming) | Improvement / Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Total Requests** | 55,915 | 103,061 | 307,483 | 58,844 | **7,535** | 🚀 **Multi-Service Kafka Fan-Out** |
| **Success Rate** | 100.00% | 100.00% | 100.00% | 100.00% | **100.00%** (7,535 / 7,535) | 🎯 **100.00% Zero-Error Scale** |
| **Median Read Latency (p50)** | 186.84 ms | 156.99 ms | 215.69 ms | 1.00 ms | **14.00 ms** | ⚡ **Offloaded to Replica DB** |
| **Atomic Outbox Orders**| N/A | N/A | N/A | N/A | **3,738 orders** | 🛡️ **100% Zero Data Loss (Postgres ACID)** |
| **95th Percentile Read (p95)** | 574.92 ms | 371.52 ms | 452.68 ms | 21.00 ms | **35.00 ms** | 🟢 **Sub-35ms Read Tail** |
| **Kafka Event Delivery** | N/A | N/A | N/A | N/A | **100% Delivered** | 📡 **Multi-Consumer Group Ingestion** |

*Saved live raw metrics to [`docs/v3_hetzner_benchmark_results.md`](docs/v3_hetzner_benchmark_results.md), [`docs/v4_benchmark_results.md`](docs/v4_benchmark_results.md), and [`docs/v6_benchmark_results.md`](docs/v6_benchmark_results.md).*


---

## Key Files

| File | Purpose |
|------|---------|
| [`src/main.rs`](src/main.rs) | Startup orchestration |
| [`src/db/mod.rs`](src/db/mod.rs) | PgPool (max 10 connections) |
| [`src/repositories/product_repository.rs`](src/repositories/product_repository.rs) | All SQL, including `decrement_inventory` with atomic update pattern |
| [`migrations/001_initial.sql`](migrations/001_initial.sql) | 5 tables: users, products, orders, order_items, flash_sales |
| [`docs/architecture/v1.md`](docs/architecture/v1.md) | Architecture diagram + concurrency model |
| [`docs/performance.md`](docs/performance.md) | Metrics reference + bottleneck investigation guide |
| [`docs/decisions/001-postgresql.md`](docs/decisions/001-postgresql.md) | Why monolith first |

---

## Historical Sessions Log

### Session 1 — 2026-08-21 (V1 Build)

**Goal:** Complete all 14 steps of the AGENTS_GUIDE.md First Task.

**Completed:**
- Initialized Rust project (`cargo init`, Cargo.toml with all deps)
- Set up Docker Compose: PostgreSQL 15, port **5434** (5432/5433 taken by clinic app)
- Created full `src/` module tree: config, db, errors, models, repositories, services, handlers, routes, middleware
- Wrote migration `001_initial.sql` with 5 tables, UUIDs, FKs, indexes, CHECK constraints
- Implemented endpoints: `GET /health`, `GET/POST /products`, `GET /products/:id`
- Created all documentation: README, `docs/architecture/v1.md`, `docs/performance.md`, ADR-001, `load-tests/README.md`
- Fixed `cargo build` to pass clean (3 warnings, 0 errors)

**Issues Encountered & Resolved:**
1. **Migration before build**: sqlx compile-time macros require the DB tables to exist. Had to run `psql` manually first.
2. **NUMERIC → f64 type mismatch**: sqlx needs `bigdecimal` feature for `NUMERIC`. Changed price column to `FLOAT8` (documented trade-off).
3. **sqlx COALESCE/CAST still nullable**: sqlx infers nullability from expression structure, not SQL semantics. Resolved by using `FLOAT8` natively.
4. **Port 5432/5433 occupied**: clinic-app Docker containers. Moved to port **5434**.
5. **Migration idempotency**: Had to wrap `CREATE TYPE order_status` in a `DO ... EXCEPTION` block.

**Verified API Results:**
```
GET  /health              → 200  {status: ok, version: v1}   0ms
POST /products            → 201  {id, name, price: 129.99}   11ms
GET  /products?limit=5    → 200  [array]                      3ms
GET  /products/:uuid      → 200  {product}                    4ms
```

**Next Session Goal:**
Write `load-tests/v1_products.js` (k6 script), run baseline load test at 10/100/500 VUs, record results, and identify the first bottleneck. Then decide with evidence whether V2 = Redis or V2 = multiple instances.

### Session 2 — 2026-08-22 (V2 Redis Caching Layer & V1 Feature Gaps)

**Goal:** Implement V2 Redis Caching Layer and complete V1 Auth API, Orders API, and SQLx Offline Mode.

**Completed:**
- Added Redis 7 service (`ecommerce_lab_redis`, mapped to port 6380 to avoid clinic-app port conflicts)
- Integrated `deadpool-redis` connection pool configured with `max_size: 100` connections
- Implemented Cache-Aside pattern in `ProductService` & `cache_repository.rs` for `GET /products` and `GET /products/:id` (sub-millisecond reads)
- Built User Auth API: `POST /auth/register` & `POST /auth/login` with `bcrypt` (cost 10) password hashing & JWT generation/verification
- Built custom `AuthUser` Axum extractor middleware for protected route security
- Built Orders API: `POST /orders` (transactional checkout with `SELECT ... FOR UPDATE` row locks to prevent stock race conditions)
- Ran `cargo sqlx prepare` to generate `.sqlx/` query metadata for offline compilation & IDE checks
- Benchmark: Ran 500 VU k6 stress test achieving **64,617 requests**, **921.77 RPS**, **0.00% error rate**, and minimum latency of **627 µs**.

---

### Session 3 — 2026-08-26 (V3 Hetzner Cloud Multi-Node VPC Benchmark & SRE Incident Masterclass)

**Goal:** Execute full Milestone V3 multi-node Hetzner Cloud infrastructure benchmark and resolve VPC networking issues.

**Completed:**
- Provisioned 4-node Hetzner Cloud cluster (`cx23` VMs: 1 Ingress Load Balancer + 2 Axum API Workers + 1 Postgres/Redis DB) in private VPC (`10.0.1.0/24`) using Terraform inline `network {}` attachment.
- Diagnosed and resolved secondary interface `enp7s0` boot state issue by injecting Netplan overlay (`/etc/netplan/60-vpc.yaml` with `dhcp4: true`) via `user_data`.
- Applied loose Reverse Path Filtering (`net.ipv4.conf.all.rp_filter = 2`) across all dual-homed instances to eliminate 504 Gateway Timeouts caused by asymmetric VPC return routing.
- Sustained **4,931 RPS** (324,951 total requests) with **100.00% success rate** under 3,000 VU load test.
- Documented full masterclass diagnostic guide in [`docs/scenarios/sre_502_504_diagnostic_scenario.md`](docs/scenarios/sre_502_504_diagnostic_scenario.md) and [`docs/mistakes/hetzner_v3_benchmark_mistakes.md`](docs/mistakes/hetzner_v3_benchmark_mistakes.md).

---

### Session 4 — 2026-08-27 (V4 Asynchronous Background Job Queue & Order Decoupling Engine)

**Goal:** Empirically reproduce order checkout synchronous in-band side-effect bottleneck and implement Redis-backed background job queue decoupling.

**Completed:**
- **Empirical Failure Verification**: Simulated a 250ms synchronous side effect (email/payment/invoice) in `POST /orders`. Under 500 VUs, throughput collapsed from **2,464 RPS down to 338 RPS** (-86% drop) and p95 latency exploded to **1,400 ms** (exceeding SLA limits).
- **V4 Background Queue Engine**: Implemented `src/jobs/` module (`types.rs`, `queue.rs`, `worker.rs`) utilizing Redis list primitives (`LPUSH` / `RPOPLPUSH`) with Tokio background worker tasks (`tokio::spawn`).
- **Order Service Decoupling**: Updated `order_service::create_order` to execute fast PostgreSQL transactions in < 15ms and push `SendOrderConfirmationEmail` & `GenerateInvoice` jobs to Redis asynchronously.
- **Queue Stats Observability**: Added `GET /queue/stats` monitoring endpoint for queue depth tracking.
- **k6 Benchmark Verification**: Re-ran 500 VU stress test (`v4_order_choke_test.js`) achieving **499.72 RPS** (+47.4% throughput gain), p95 latency drop down to **818 ms** (SLA PASS ✅), 100.00% success rate across 30,200 requests, and 0 failed background queue jobs.

---

### Session 5 — 2026-08-28 (V4 Empirical Bottleneck Isolation, EXPLAIN ANALYZE & Database Sizing Math)

**Goal:** Empirically evaluate V4 cluster load testing, diagnose PostgreSQL row locking & pool queueing bottlenecks, analyze query planner cost models (`EXPLAIN ANALYZE`), and document database scaling capacity formulas.

**Completed:**
- **V4 Cluster Load Benchmarking**:
  - Ran `v4_order_choke_test.js` (500 VUs): Processed **43,866 total requests** at **721.26 RPS**, 100.00% success rate, 258ms p50 latency, and 636ms p95 latency (SLA PASS ✅).
  - Ran `v4_realistic_mixed_workload.js` (2,000 VUs): Sustained **59,163 total requests** at **551.46 RPS**, achieving **1.00ms p50 catalog read latency** and **4.18ms p95 overall latency**.
- **PostgreSQL Bottleneck Diagnosis**:
  - Isolated write tail latency spike (1.96s max) to PostgreSQL `PgPool` 10-connection queueing and `UPDATE products SET inventory_count = inventory_count - 1` `ExclusiveLock` row lock contention.
  - Tested live PostgreSQL diagnostics using `docker exec ecommerce_lab_db psql`, `pg_stat_activity`, and `EXPLAIN ANALYZE`.
- **Query Planner Mechanics & Cost-Based Optimizer (CBO)**:
  - Analyzed why PostgreSQL chooses `Seq Scan` vs. `Index Scan` / `Bitmap Index Scan`.
  - Discovered selectivity threshold rule (~15-20% table selectivity) and random disk seek I/O cost math (`seq_page_cost=1.0` vs `random_page_cost=4.0`).
- **Database Scaling Documentation**:
  - Created [`docs/study-notes-database-scaling-formulas.md`](docs/study-notes-database-scaling-formulas.md) covering market scaling tiers (Tier 1–5), Little's Law ($L = \lambda \cdot W$), PostgreSQL connection pool formula ($\text{Pool Size} = (\text{Cores} \times 2) + \text{Spindle}$), VU-to-RPS math, and updated [`docs/README.md`](docs/README.md).

---

### Session 6 — 2026-08-28 (V5 Database Scaling, Read Replicas, CQRS & Replication Lag Circuit Breaker)

**Goal:** Implement PostgreSQL Primary/Replica Physical Streaming Replication, Rust CQRS dual database pools (`writer_pool` & `reader_pool`), automated Replication Lag Circuit Breaker (`AtomicBool`), and composite B-tree database indexing.

**Completed:**
- **Docker Compose Streaming Replication Cluster**:
  - Configured `postgres` Primary container with `wal_level=replica`, `max_wal_senders=10`, `max_replication_slots=10`, and `init-primary-replication.sh` script to grant replication user permissions.
  - Added `postgres_replica` Standby container bootstrapped via `pg_basebackup` streaming replication (host port `5435`).
  - Verified live streaming replication via `SELECT client_addr, state, sync_state FROM pg_stat_replication;` showing active `walreceiver` streaming.
- **Rust CQRS Read-Write Splitting**:
  - Refactored `AppConfig` and `src/db/mod.rs` to initialize dual pools: `DbPools { writer: PgPool, reader: PgPool }`.
  - Updated handlers (`products.rs`, `orders.rs`, `auth.rs`) to route 100% of read queries (`GET /products`, `GET /orders`, `POST /auth/login`) to `reader_pool` and mutations/transactions to `writer_pool`.
- **Replication Lag Circuit Breaker**:
  - Added `check_replication_lag` helper querying `pg_stat_replication` LSN delta in milliseconds.
  - Spawned background Tokio task checking lag every 1 second and updating `is_replica_lagging` `AtomicBool` flag.
  - Added `GET /health/db` endpoint reporting pool connections, idle counts, replication lag, and circuit breaker status.
- **Composite Indexing & Query Planner Execution**:
  - Added migration `002_v5_indexes.sql` creating composite index `idx_products_name_price ON products(name, price)`.
  - Verified via `EXPLAIN ANALYZE` achieving **0.046 ms** execution time with `Index Scan`.

---

### Session 7 — 2026-08-29 (PostgreSQL Streaming Replication Infrastructure Deep-Dive Masterclass)

**Goal:** Conduct a comprehensive first-principles deep dive into PostgreSQL Physical Streaming Replication infrastructure (`init-primary-replication.sh`, `start-replica.sh`), WAL receiver/sender background process loop, CQRS dual connection pools, and Cloud VPC production security practices.

**Completed:**
- **Script Line-by-Line Breakdown**:
  - Analyzed [`scripts/init-primary-replication.sh`](scripts/init-primary-replication.sh) (`pg_hba.conf` `replication` host permission, `WITH REPLICATION` user creation, `SELECT pg_reload_conf()`).
  - Analyzed [`scripts/start-replica.sh`](scripts/start-replica.sh) (`pg_isready` readiness polling, `$PGDATA` initialization checks, `rm -rf "${PGDATA:?}"/*` safety guards, `pg_basebackup -R` standby signal & connection auto-config generation, `exec` PID 1 process replacement).
- **Internal Database Engine Loop Mechanics**:
  - Evaluated how `pg_basebackup -R` generates `standby.signal` and `postgresql.auto.conf`.
  - Traced PostgreSQL kernel processes: `walreceiver` process on Replica connecting via persistent TCP socket to `walsender` process on Primary, continuously fetching and replaying WAL binary logs in a 24/7 background loop.
- **Production Cloud VPC Architecture**:
  - Compared local Docker bridge networking (`postgres` DNS resolution) vs. Cloud VPC Production (`10.0.x.x` Private Subnets, Security Groups, `hostssl`, `scram-sha-256`, and TLS encryption in transit).
- **Study Notes Artifacts**:
  - Created [`docs/study-notes-postgresql-streaming-replication.md`](docs/study-notes-postgresql-streaming-replication.md) consolidating all replication concepts, code breakdowns, and cloud SRE guidelines.
  - Created [`docs/study-notes-read-your-own-writes-sticky-sessions.md`](docs/study-notes-read-your-own-writes-sticky-sessions.md) detailing distributed consistency race conditions, timeline diagrams, and Redis sticky session routing in Rust.

---

### Session 8 — 2026-08-29 (Milestone V6: Transactional Outbox Pattern & Apache Kafka Event Streaming)

**Goal:** Implement PostgreSQL Transactional Outbox Pattern, Apache Kafka (Redpanda) event producer/relay, and independent Consumer Groups in Rust.

**Completed:**
- **Infrastructure & Migration**:
  - Created migration `migrations/003_v6_outbox.sql` for the `outbox` table with pending index.
  - Added Redpanda Kafka broker container (`ecommerce_lab_redpanda`) to `docker-compose.yml` (ports 9092 / 29092).
- **Atomic Outbox Persistence**:
  - Updated `order_repository::create_order` to write `Order` and `OrderCreated` outbox payload in the same atomic `BEGIN...COMMIT` PostgreSQL transaction.
  - Removed direct Redis queue calls from `order_service.rs` (order checkout path is 100% atomic in PostgreSQL).
- **Outbox Relay & Kafka Consumer Groups**:
  - Implemented Outbox Relay Worker (`src/events/outbox_relay.rs`) polling pending outbox rows using `SKIP LOCKED`, producing to Kafka topic `ecom-order-events`, and updating `status = 'processed'`.
  - Implemented 2 independent Consumer Groups (`notification-service-group` and `analytics-service-group`) in `src/events/consumers.rs`.
- **E2E & Live System Verification**:
  - Verified `POST /orders` creates an order + outbox event atomically.
  - Confirmed Outbox Relay published event to Kafka, both Consumer Groups ingested offset 0, and PostgreSQL `outbox` record was marked `status = 'processed'` with timestamp `processed_at`.
- **Study Notes**:
  - Created [`docs/study-notes-v6-outbox-kafka-architecture.md`](docs/study-notes-v6-outbox-kafka-architecture.md) detailing outbox design, Kafka replayability, and trade-offs.

### Session 9 — 2026-08-30 (Milestone V7: Docker & Kubernetes Orchestration, cargo-chef & Local K3s Cluster)

**Goal:** Evolve local container infrastructure into a production-grade Kubernetes cluster (`k3d`), implementing multi-stage container optimization via `cargo-chef`, health probes, stateful services, and declarative manifests.

**Completed:**
- **Container Build Optimization (`cargo-chef`)**:
  - Upgraded [Dockerfile](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/Dockerfile) to a 3-stage `cargo-chef` setup caching Rust dependency layers across builds.
  - Added [.dockerignore](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/.dockerignore) excluding local 6GB `target/` build folder (reclaiming 10.64GB Docker disk space).
- **Declarative K8s Manifest Tree (`infrastructure/k8s/`)**:
  - `00-namespace.yaml`: Created `ecommerce-lab` namespace.
  - `01-configmap-secrets.yaml`: Declarative `ConfigMap` and `Secret` objects.
  - `02-postgres.yaml`: `Deployment` & `Service` for Pg Primary (5432) and Read Replica (5435) with resource limits.
  - `03-redis.yaml`: `Deployment` & `Service` for Redis 7 (6379).
  - `04-redpanda.yaml`: `Deployment` & `Service` for Redpanda Kafka broker (9092).
  - `05-axum-api.yaml`: `Deployment` (3 replicas) for Axum API with `livenessProbe` (`GET /health`) & `readinessProbe` (`GET /health/db`).
  - `06-ingress.yaml`: Ingress controller routing external HTTP requests to `axum-api-service`.
- **Local K3s Automation & Verification**:
  - Created [deploy-k3d.sh](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/scripts/deploy-k3d.sh) and [teardown-k3d.sh](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/scripts/teardown-k3d.sh).
  - Configured custom `eviction-hard < 2%` K3s flags to handle host disk space constraints cleanly.
  - Verified 100% pod readiness (`3/3 axum-api`, `1/1 postgres-primary`, `1/1 postgres-replica`, `1/1 redis`, `1/1 redpanda`).
  - Verified live E2E HTTP endpoints: `curl http://localhost:8888/health` (200 OK), `curl http://localhost:8888/health/db` (200 OK), `POST /products` (201 Created), and `GET /products` (200 OK).

---


## Resume & Interview Case Study Plan


> *Plan for polishing this project as a high-impact portfolio case study after scaling milestones are complete.*

### GitHub Repo Title:
`rust-high-throughput-ecommerce-lab`

### Resume Title:
**High-Throughput E-Commerce Systems Engine in Rust** | *Rust, Axum, Tokio, PostgreSQL, k6, Playwright*

### Key Resume Bullet Points:
- **Architecture**: Designed a high-concurrency REST API in **Rust (Axum + Tokio)** using layered architecture (Service/Repository pattern) and compile-time verified SQL queries via `sqlx`.
- **Benchmarking**: Conducted load testing with **k6**, achieving **1,220+ Requests/Sec (RPS)** with **67.89ms p95 latency** and **0.00% error rate** under 100 concurrent VUs.
- **Performance Engineering**: Identified database connection pool queuing under 500 VUs using **Little's Law**; diagnosed PostgreSQL process thrashing when pool size was raised to 50 connections.
- **Automated E2E Testing**: Developed an automated End-to-End API test suite using **Playwright** covering full product CRUD lifecycles and HTTP status validation.
- **Target V2-V10 Case Study Additions**: Benchmark Redis caching (V2), multi-instance load balancing (V3), background job queue (V4), read replicas (V5), and K8s autoscaling (V7-V10).

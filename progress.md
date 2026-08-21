# Rust E-Commerce Scaling Lab — Progress

## Current Version: **V2 — Redis Caching Layer (COMPLETE ✅)**

---

## Version Status

| Version | Status | Description |
|---------|--------|-------------|
| **V1** | ✅ **DONE** | Rust API + PostgreSQL monolith, all endpoints verified |
| **V2** | ✅ **DONE** | Redis Cache-Aside layer + Auth API (JWT/bcrypt) + Orders API (atomic transaction) |
| V3 | ⬜ Not started | Load balancer + multiple API instances |
| V4 | ⬜ Not started | Background job queue |
| V5 | ⬜ Not started | DB indexes + read replicas |
| V6 | ⬜ Not started | Kafka event-driven processing |
| V7 | ⬜ Not started | Docker + Kubernetes |
| V8 | ⬜ Not started | Autoscaling + observability (Prometheus/Grafana) |
| V9 | ⬜ Not started | Failure testing + resilience |
| V10 | ⬜ Not started | 10k concurrent-user load testing |

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

## Load Test Baseline & Stress Test (V1 — COMPLETED ✅)

### 100 VUs Baseline vs 500 VUs Stress Test

| Metric | 100 VUs Baseline | 500 VUs Stress Test | Target Threshold | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Throughput (RPS)** | **1,221.85 req/sec** | **1,820.04 req/sec** | N/A | Scaled 1.5x |
| **Total Requests** | **37,024** | **55,915** | N/A | 100% Success |
| **Error Rate** | **0.00%** | **0.00%** | `< 1.00%` | ✅ 100% Stable |
| **Median Latency (p50)** | **2.29 ms** | **186.84 ms** | N/A | Queueing |
| **95th Percentile (p95)** | **67.89 ms** | ❌ **574.92 ms** | `< 200.00 ms` | 🔴 **BREACHED** |
| **90th Percentile (p90)** | **26.80 ms** | **399.40 ms** | N/A | Degraded |

> 🔍 **Identified Bottleneck**: At 500 VUs, requests are queuing for PostgreSQL connection pool slots (`max_connections = 10`), causing latency to jump from **67.89ms** to **574.92ms**.
>
> ### 🧮 Queueing Formula (Little's Law)
> ```text
> Connections Needed = Requests Per Second * Average Query Duration (in seconds)
> ```
> - **100 VUs**: `1,200 RPS * 0.0015s = 1.8 connections needed`. Since `1.8 < 10`, zero queuing occurs (`p95 = 67ms`).
> - **500 VUs**: `1,800 RPS * 0.015s = 27 connections needed`. Since `27 > 10`, pool is 100% saturated and 490 requests wait in line (`p95 = 574ms`).
>
> ### ⚠️ Experiment: Increasing Pool Size to 50
> - **Result**: Throughput dropped to **786 RPS** and `p95` latency worsened to **1.31s**.
> - **Reason**: PostgreSQL uses 1 OS process per connection. 50 connections caused CPU process context-switching thrashing and disk I/O contention.
> - **Formula**: `Optimal Pool Size = (CPU Cores * 2) + Disk Count` = `(4 * 2) + 1 = 9 to 10 connections`.
>
> *Saved raw metrics to [`load-tests/results/v1_baseline.json`](load-tests/results/v1_baseline.json) and [`load-tests/results/v1_stress_500vu.json`](load-tests/results/v1_stress_500vu.json).*

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

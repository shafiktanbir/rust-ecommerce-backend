# Rust E-Commerce Scaling Lab — Progress

## Current Version: **V3 — Docker Load-Balanced Cluster (COMPLETE ✅)**

---

## Version Status

| Version | Status | Description |
|---------|--------|-------------|
| **V1** | ✅ **DONE** | Rust API + PostgreSQL monolith, all endpoints verified |
| **V2** | ✅ **DONE** | Redis Cache-Aside layer + Auth API (JWT/bcrypt) + Orders API (atomic transaction) |
| **V3** | ✅ **DONE** | Docker Compose multi-instance cluster + Nginx Load Balancer (3x Axum workers) |
| V4 | ⬜ Not started | Background job queue |
| V5 | ⬜ Not started | DB indexes + read replicas |
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

## Load Test Baseline & Stress Test (V1 vs V2 vs V3 — COMPLETED ✅)

### Multi-Milestone Benchmark Comparison

| Metric | V1 500 VUs (Pg Monolith) | V2 500 VUs (Redis Cache-Aside) | V3 3,000 VUs (Docker Local Cluster) | V3 3,000 VUs (Hetzner Live Cloud Cluster) | Improvement / Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Total Requests** | 55,915 | 103,061 | 234,165 | **307,483** | 🚀 **+31% Capacity vs Local Cluster** |
| **Throughput (RPS)** | 1,820.04 req/sec | 1,469.69 req/sec | 2,464.35 req/sec | **4,665.84 req/sec** | 🚀 **+89% Higher RPS on Live VPS** |
| **Success Rate** | 100.00% | 100.00% | 99.95% | **100.00%** (307,483 / 307,483) | 🎯 **100.00% Zero-Error Scale** |
| **Min Latency** | ~1.50 ms | 360.63 µs | 241.80 µs | **174.10 ms** | Live VPC Network Latency |
| **Median Latency (p50)** | 186.84 ms | 156.99 ms | 277.07 ms | **215.69 ms** | ⚡ **-22% Faster Than Local Docker** |
| **90th Percentile (p90)** | 399.40 ms | 314.08 ms | 567.04 ms | **334.61 ms** | 🟢 **Smooth Tail Distribution** |
| **95th Percentile (p95)** | 574.92 ms | 371.52 ms | 679.37 ms | **452.68 ms** | 🟡 **Sub-500ms Under 3,000 VUs** |

> 🔍 **V3 Hetzner Live Cloud Cluster & SRE Architecture Analysis**:
> 1. **Inline Declarative Network Pattern**: Transitioning from separate `hcloud_server_network` hotplug resources to inline `network { network_id = ... ip = "10.0.1.x" }` inside `hcloud_server` eliminated boot race conditions and allowed instant 200 OK health verification.
> 2. **IP Quota & Security Hardening**: Operating internal worker nodes (`10.0.1.11`, `10.0.1.12`) and database node (`10.0.1.20`) with `ipv4_enabled = false` isolated backend storage inside the private VPC while keeping IPv4 usage to 1 single Ingress Load Balancer IP.
> 3. **4,665 RPS Sustained Capacity**: The 4-node Hetzner cluster (`cx23` VMs) sustained **4,665.84 RPS** with **100.00% success rate** across 307,483 requests under 3,000 VU peak load.

*Saved live raw metrics to [`docs/v3_hetzner_benchmark_results.md`](docs/v3_hetzner_benchmark_results.md) and infrastructure guide to [`infrastructure/v3-nginx-cluster/README.md`](infrastructure/v3-nginx-cluster/README.md).*

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


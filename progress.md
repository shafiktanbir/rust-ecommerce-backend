# Rust E-Commerce Scaling Lab — Progress

## Current Version: **V1 — Monolith (COMPLETE ✅)**

---

## Version Status

| Version | Status | Description |
|---------|--------|-------------|
| **V1** | ✅ **DONE** | Rust API + PostgreSQL monolith, all endpoints verified |
| V2 | ⬜ Not started | Add Redis cache |
| V3 | ⬜ Not started | Load balancer + multiple API instances |
| V4 | ⬜ Not started | Background job queue |
| V5 | ⬜ Not started | DB indexes + read replicas |
| V6 | ⬜ Not started | Kafka event-driven processing |
| V7 | ⬜ Not started | Docker + Kubernetes |
| V8 | ⬜ Not started | Autoscaling + observability (Prometheus/Grafana) |
| V9 | ⬜ Not started | Failure testing + resilience |
| V10 | ⬜ Not started | 10k concurrent-user load testing |

---

## V1 Confidence Ratings (1–5)

| Topic | Confidence | Notes |
|-------|-----------|-------|
| Axum handler/router setup | 3 | Built it, need to internalize extractor patterns |
| Tokio async runtime model | 3 | Understand conceptually, not hands-on debugged yet |
| sqlx compile-time verification | 4 | Hit it in practice, understand the mechanism |
| Connection pool mechanics | 3 | Know the math (pool_size = rps / (1000/query_ms)) |
| Repository pattern (SQL isolation) | 4 | Clear separation enforced |
| Service / Handler layering | 3 | Implemented, not deeply tested yet |
| PostgreSQL schema design (FK, CHECK, UUID) | 4 | Applied correctly |
| Docker Compose setup | 4 | Ran it, hit port conflicts, resolved them |
| Migration idempotency (IF NOT EXISTS, DO block) | 4 | Hit it in practice and fixed it |
| NUMERIC vs FLOAT8 in sqlx | 4 | Discovered the trade-off, documented it |

---

## V1 Gaps / Things to Revisit

- [ ] **Auth layer**: `User` model exists but register/login endpoints not implemented
- [ ] **Error 404 verification**: `GET /products/:invalid-uuid` not tested
- [ ] **Validation errors**: test `POST /products` with bad payload (negative price, empty name)
- [ ] **rust-analyzer offline mode**: `cargo sqlx prepare` not yet run (IDE shows false errors)
- [ ] **FLOAT8 vs NUMERIC**: understand when to switch to `rust_decimal` + `bigdecimal`
- [ ] **Orders API**: not implemented yet (tables exist in DB)

---

## Architectural Decisions Made

| ADR | Decision | Reason |
|-----|----------|--------|
| [ADR-001](docs/decisions/001-postgresql.md) | V1 = intentional monolith | No measurements yet → no reason to add complexity |

---

## Load Test Baseline (V1 — not yet run)

> Run these after V1 is stable to establish the baseline before V2.

```bash
# Install k6 first (see load-tests/README.md)
k6 run --vus 10  --duration 30s load-tests/v1_products.js
k6 run --vus 100 --duration 60s load-tests/v1_products.js
k6 run --vus 500 --duration 60s load-tests/v1_products.js
```

Record results in `load-tests/results/` using the template in `docs/performance.md`.

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

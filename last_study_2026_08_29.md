# Last Study Session Summary — 2026-08-29

> **Topic**: Milestone V5: PostgreSQL Physical Streaming Replication, CQRS Read-Write Splitting, Replication Lag Circuit Breaker, and Composite Indexing.

---

## 1. What We Built & Accomplished

### A. PostgreSQL Physical Streaming Replication
- **Primary Setup**: Configured `postgres` with WAL archiving (`wal_level=replica`, `max_wal_senders=10`) and granted replication privileges to `replicator` user (`scripts/init-primary-replication.sh`).
- **Replica Setup**: Configured `postgres_replica` Standby container initialized via `pg_basebackup` streaming replication on host port `5435` (`scripts/start-replica.sh`).
- **Live Replication**: Confirmed active streaming status via `SELECT client_addr, state, sync_state FROM pg_stat_replication;` (`walreceiver` streaming in `async` mode).

### B. Rust CQRS Read-Write Splitting Architecture
- **Dual Pools**: Initialized `DbPools { writer: PgPool, reader: PgPool }` in `src/db/mod.rs` & `src/config/mod.rs`.
- **Dynamic Routing**:
  - `POST /products`, `POST /orders`, `POST /auth/register` $\rightarrow$ Routed to `writer_pool` (Primary DB).
  - `GET /products`, `GET /products/:id`, `GET /orders`, `POST /auth/login` $\rightarrow$ Routed to `reader_pool` (Replica DB).

### C. Automated Replication Lag Circuit Breaker
- Background Tokio task in `src/main.rs` checks `pg_stat_replication` LSN delta every 1 second.
- **Circuit Breaker Logic**:
  - If `lag > 500ms`: Automatically trips circuit breaker (`is_replica_lagging` `AtomicBool`), routing 100% of reads to Primary DB to protect data freshness.
  - If `lag < 100ms`: Automatically recovers and resumes sending reads to Replica DB.
- **Health Check**: Added `GET /health/db` endpoint returning live pool metrics, replication lag ms, and circuit breaker state.

### D. Composite B-Tree Indexing & Query Execution
- Created migration `migrations/002_v5_indexes.sql` with composite index `idx_products_name_price ON products(name, price)`.
- Verified via `EXPLAIN ANALYZE`: **`0.046 ms`** query execution time using `Index Scan`.

---

## 2. Core Concepts to Revise & Remember

1. **WAL (Write-Ahead Logging)**:
   - PostgreSQL appends changes sequentially to WAL before flushing dirty data to disk table files.
   - Streaming replication transfers raw WAL binary bytes over TCP to replicas.
2. **Asynchronous vs Synchronous Replication**:
   - **Async (Default)**: Fast writes (0.5ms), microsecond lag window (2–50ms).
   - **Sync**: Zero data loss guarantee, but write latency multiplies by network RTT.
3. **CQRS / Read-Write Splitting**:
   - Offloads 90% of catalog browse traffic from Primary to Replicas.
   - Prevents read queries from consuming Primary CPU cores and connection pools during flash sales.
4. **Replication Lag Circuit Breaker**:
   - Prevents "Read-Your-Own-Writes" stale data bugs when replica falls behind during network strain.
   - Uses zero-overhead `AtomicBool` flag in Rust app state (`std::sync::atomic::Ordering::Relaxed`).
5. **Leftmost Prefix Rule in Composite Indexing**:
   - An index on `(name, price)` speeds up queries on `(name)` and `(name, price)`, but CANNOT serve queries filtering ONLY on `(price)`.

---

## 3. Verification & Live Metrics

| Test / Endpoint | Result | Description |
| :--- | :--- | :--- |
| `cargo check` & `cargo build --release` | ✅ PASS | Zero errors, clean compilation |
| `docker compose up -d` | ✅ PASS | All 13 cluster containers UP and healthy |
| Streaming Replication | ✅ PASS | Active `walreceiver` streaming in `async` mode |
| `GET /health/db` | ✅ PASS | Returns pool size, replication lag ms, circuit breaker status |
| `POST /products` & `GET /products` | ✅ PASS | Product created on Primary, read from Replica |
| `EXPLAIN ANALYZE` Index Scan | ✅ PASS | Sub-millisecond composite B-tree index lookup (0.046 ms) |

---

## 4. Git Commits
- [`98d8b0c`](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/progress.md): `docs: add database scaling capacity formulas and Session 5 progress notes`
- [`c2dd50a`](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/progress.md): `feat(v5): implement PostgreSQL read replicas, CQRS read-write splitting, lag circuit breaker, and composite indexes`

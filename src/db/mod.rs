// src/db/mod.rs
//
// WHY THIS EXISTS:
//   Every HTTP request needs to talk to PostgreSQL.
//   Naively, you might open a new TCP connection per request. This is catastrophically expensive:
//
//     - TCP handshake:         ~1ms
//     - TLS handshake:         ~5ms
//     - PostgreSQL auth:       ~2ms
//     - Total per request:     ~8ms just to connect (before any SQL runs)
//
//   At 1,000 req/s that wastes 8,000ms/s of CPU on connection overhead alone.
//
//   The solution is a CONNECTION POOL: keep a set of already-connected connections
//   alive and re-use them across requests.
//
// HOW THE POOL WORKS:
//
//   [Request 1]  ─┐
//   [Request 2]  ─┤─→  Pool (10 idle connections) ─→  PostgreSQL
//   [Request 3]  ─┘
//
//   Each request borrows a connection from the pool, uses it, then returns it.
//   The pool manages the connection lifecycle.
//
// WHAT HAPPENS AT SCALE:
//
//   10 requests   → Pool has 10 connections, all serving simultaneously — fine
//   100 requests  → Pool has 10 connections, 90 requests wait in queue — fine if queries are fast
//   1,000 req/s   → If each query takes 10ms, each connection can serve 100/s → need 10 connections
//   10,000 req/s  → Need 100 connections, but PostgreSQL max_connections is ~100–200 by default
//                   → This is where V1 starts to break — this becomes the first bottleneck we'll measure
//
// POOL SIZE MATH:
//   pool_size = target_rps / (1000ms / avg_query_latency_ms)
//   Example: 1000 rps, 10ms queries → pool_size = 1000 / 100 = 10 connections
//
// HOW A SENIOR ENGINEER DIAGNOSES POOL PROBLEMS:
//
// ── TOOL 1: Application logs (first thing to check) ──────────────────────────
//
//   When pool is exhausted, sqlx logs this error:
//     ERROR ecommerce_lab: PoolTimedOut: timed out waiting for connection from pool
//
//   Or you see requests taking suspiciously long:
//     DEBUG request{method=GET uri=/products}: finished latency=8200ms status=500
//                                                                ↑ this is the queue wait time
//
// ── TOOL 2: pg_stat_activity (the senior engineer's X-ray for PostgreSQL) ────
//
//   pg_stat_activity is a LIVE VIEW built into PostgreSQL.
//   One row = one open connection. It shows what each connection is doing RIGHT NOW.
//   You can query it from DBeaver, psql, or any SQL client while the app is running.
//
//   KEY COLUMNS:
//     pid             → process ID (PostgreSQL's internal ID for this connection)
//     state           → what the connection is doing right now:
//                         "idle"   = connected, no query running (free in pool)
//                         "active" = query currently executing (busy)
//                         "idle in transaction" = inside BEGIN but paused (danger sign)
//     wait_event_type → WHY it's waiting:
//                         "Client"  = waiting for next request (normal for idle pool connections)
//                         "Lock"    = blocked by another query's row lock (flash sale problem)
//                         "IO"      = waiting for disk (slow query)
//     query_start     → when the current query started (use now()-query_start for duration)
//     query           → the actual SQL text running right now
//     application_name → which app opened this connection ("ecommerce_lab", "psql", etc)
//
//   QUERY 1 — Health snapshot (run this first):
//     SELECT state, count(*) FROM pg_stat_activity
//     WHERE datname = 'ecommerce_lab' GROUP BY state;
//
//     Healthy output:       Exhausted pool output:
//       state  | count        state  | count
//       -------+------        -------+------
//       idle   |   9   ✅     active |  10   🔴  (all connections busy)
//       active |   1          (no idle rows = queue building in Rust)
//
//   QUERY 2 — What queries are running RIGHT NOW:
//     SELECT pid, state, now()-query_start AS duration, left(query, 80) AS query
//     FROM pg_stat_activity
//     WHERE datname = 'ecommerce_lab' AND state = 'active'
//     ORDER BY duration DESC;
//
//     → Sort by duration DESC to see the SLOWEST query at the top.
//     → If duration > 100ms for a simple SELECT: missing index, table scan.
//     → If duration > 1s: lock contention or extremely slow query.
//
//   QUERY 3 — Detect lock contention (flash sale bottleneck):
//     SELECT pid, state, wait_event_type, wait_event, left(query, 80) AS query
//     FROM pg_stat_activity
//     WHERE datname = 'ecommerce_lab' AND wait_event_type = 'Lock'
//     ORDER BY pid;
//
//     → Rows appearing here = connections waiting for a row lock to release.
//     → During flash sale under load: UPDATE products SET inventory_count...
//       will show multiple pids all waiting for the same row lock.
//     → This is the EXACT bottleneck we'll hit in the decrement_inventory function.
//
//   QUERY 4 — Danger sign: "idle in transaction" connections:
//     SELECT pid, state, now()-state_change AS stuck_for, left(query, 80) AS query
//     FROM pg_stat_activity
//     WHERE datname = 'ecommerce_lab' AND state = 'idle in transaction'
//     ORDER BY stuck_for DESC;
//
//     → "idle in transaction" means: BEGIN was called, query ran, but COMMIT/ROLLBACK
//       never happened. The connection holds ROW LOCKS until it commits.
//     → This starves other connections. A bug that opens transactions and doesn't close
//       them will cause ALL other queries to queue behind those held locks.
//     → In production: if you see these stuck > 30s, something is very wrong.
//
//   QUERY 5 — Total connections vs PostgreSQL's limit:
//     SELECT count(*) AS total_connections,
//            (SELECT setting::int FROM pg_settings WHERE name='max_connections') AS pg_max
//     FROM pg_stat_activity;
//
//     → PostgreSQL default max_connections = 100.
//     → If total_connections approaches max: new connections are REFUSED.
//     → At scale: 3 app servers × 10 pool connections = 30 (fine).
//                 3 app servers × 50 pool connections = 150 > 100 (PostgreSQL refuses).
//                 Solution: PgBouncer (connection pooler at DB level) — a V5+ concern.
//
// ── TOOL 3: k6 load test (confirms what pg_stat_activity shows) ──────────────
//
//   Run k6 while watching pg_stat_activity in DBeaver side by side.
//   When k6 shows p95 spiking from 10ms → 800ms:
//     → Switch to DBeaver
//     → Run QUERY 1: if idle=0, active=10 → pool exhausted → increase max_connections
//     → Run QUERY 3: if Lock rows appear → row lock contention → redesign flash sale logic
//
//   Senior engineer mental model:
//     p95 spike + pool exhausted   → increase pool OR optimize query speed
//     p95 spike + lock contention  → redesign the locking strategy
//     p95 spike + slow queries     → add indexes, use EXPLAIN ANALYZE to find table scans

use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::config::AppConfig;

/// Create and validate the PostgreSQL connection pool.
///
/// This function is async because establishing the initial connections requires network I/O.
/// `PgPoolOptions` lets us tune the pool before connections are made.
pub async fn create_pool(config: &AppConfig) -> PgPool {
    PgPoolOptions::new()
        // V1 starting point: 10 connections
        // We will measure and tune this during load testing
        .max_connections(10)
        // How long to wait for a connection before returning an error
        // If the pool is exhausted, requests will wait up to 30s
        .acquire_timeout(std::time::Duration::from_secs(30))
        // Test the connection immediately — fail fast if DB is unreachable
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to PostgreSQL. Is Docker running? Is DATABASE_URL correct?")
}

/// Create and validate the Redis connection pool for V2 caching.
pub fn create_redis_pool(config: &AppConfig) -> deadpool_redis::Pool {
    let mut redis_config = deadpool_redis::Config::from_url(&config.redis_url);
    redis_config.pool = Some(deadpool_redis::PoolConfig::new(100));
    redis_config
        .create_pool(Some(deadpool_redis::Runtime::Tokio1))
        .expect("Failed to create Redis connection pool")
}

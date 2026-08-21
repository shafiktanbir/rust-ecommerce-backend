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
// HOW WE'LL MEASURE THIS LATER:
//   - "waiting for connection from pool" in tracing logs
//   - pg_stat_activity shows connections and their states
//   - k6 will show p95 latency spiking when pool is exhausted

use sqlx::{PgPool, postgres::PgPoolOptions};

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

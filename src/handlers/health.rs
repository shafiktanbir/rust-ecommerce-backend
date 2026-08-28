// src/handlers/health.rs
//
// WHY A HEALTH ENDPOINT:
//   Every production system needs a health check endpoint.
//   Load balancers (nginx, ALB, K8s) probe this endpoint to know if the instance
//   is ready to receive traffic.
//
//   Without it:
//     - A restarting instance receives traffic before it's ready → errors for users
//     - You cannot know from outside if the service is alive
//
//   V1: Simple health check — just confirms the HTTP server is running.
//   V2+: We'll add database connectivity check and version info.
//
// RUST CONCEPT — Handler signature in Axum:
//   Axum handlers are async functions that return `impl IntoResponse`.
//   Axum uses a trait called `Handler` that is automatically implemented for
//   any async function whose arguments are all extractors (Query, Json, State, Path, etc.)
//   and whose return type implements IntoResponse.
//   This zero-cost abstraction happens entirely at compile time.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::{db::check_replication_lag, routes::AppState};

/// GET /health
///
/// Returns: 200 OK with basic service info
pub async fn health_check() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "version": "v5",
            "service": "ecommerce-lab"
        })),
    )
}

/// GET /health/db
///
/// Returns: 200 OK with database connection pool metrics, replication lag in ms, and circuit breaker status
pub async fn db_health_check(State(state): State<AppState>) -> impl IntoResponse {
    let lag_ms = check_replication_lag(&state.db.writer).await;
    let is_lagging = state.is_replica_lagging.load(std::sync::atomic::Ordering::Relaxed);

    (
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "writer_pool_connections": state.db.writer.size(),
            "writer_pool_idle": state.db.writer.num_idle(),
            "reader_pool_connections": state.db.reader.size(),
            "reader_pool_idle": state.db.reader.num_idle(),
            "replication_lag_ms": lag_ms,
            "circuit_breaker_replica_lagging": is_lagging
        })),
    )
}


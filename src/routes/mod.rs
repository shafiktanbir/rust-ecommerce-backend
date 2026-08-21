// src/routes/mod.rs
//
// WHY THIS EXISTS:
//   The routes module assembles the complete Axum Router tree.
//   It also defines AppState — the shared state cloned into every handler.
//
// RUST CONCEPT — Arc and AppState:
//   AppState derives Clone. Axum clones it for each request.
//   PgPool is Arc<Inner> internally, so cloning AppState just increments an
//   atomic reference count — no data is copied.
//   This is safe to share across concurrent requests without any locking.
//
//   If AppState had data that required mutation (e.g. a counter), we would use
//   Arc<Mutex<Counter>>. For read-only shared data, Arc alone suffices.
//
// RUST CONCEPT — Trait bounds (Clone + Send + Sync):
//   Axum requires AppState to implement Clone + Send + Sync because:
//     - Clone: each request clone its own state reference
//     - Send: the state might be moved between threads in the Tokio thread pool
//     - Sync: multiple threads can access it simultaneously
//   PgPool satisfies all of these.

use axum::{
    routing::{get, post},
    Router,
};
use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;

use crate::{
    config::AppConfig,
    handlers::{auth, health, orders, products},
};

/// Shared application state — injected into every handler via State<AppState>
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis: RedisPool,
    pub config: AppConfig,
}

/// Build the complete application router with all routes registered.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/health", get(health::health_check))
        // Auth
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        // Products
        .route("/products", get(products::list_products))
        .route("/products", post(products::create_product))
        .route("/products/:id", get(products::get_product))
        // Orders
        .route("/orders", post(orders::create_order))
        .route("/orders", get(orders::list_orders))
        .route("/orders/:id", get(orders::get_order))
        // Attach shared state
        .with_state(state)
}

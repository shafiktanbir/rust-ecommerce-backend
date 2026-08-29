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

use std::sync::{atomic::AtomicBool, Arc};

use axum::{
    routing::{get, post},
    Router,
};
use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;

use crate::{
    config::AppConfig,
    db::DbPools,
    handlers::{auth, health, orders, products, queue_stats},
};

use deadpool_redis::redis::AsyncCommands;

/// Shared application state — injected into every handler via State<AppState>
#[derive(Clone)]
pub struct AppState {
    pub db: DbPools,
    pub redis: RedisPool,
    pub config: AppConfig,
    pub is_replica_lagging: Arc<AtomicBool>,
}

impl AppState {
    /// Get the appropriate pool for read operations.
    /// Uses read replica when healthy; falls back to writer pool if circuit breaker trips.
    pub fn get_reader_pool(&self) -> &PgPool {
        if self.is_replica_lagging.load(std::sync::atomic::Ordering::Relaxed) {
            &self.db.writer
        } else {
            &self.db.reader
        }
    }

    /// Read-Your-Own-Writes Sticky Session Pool Switcher:
    /// Checks if a specific user/session has recently performed a write operation (within sticky TTL window).
    /// If sticky flag is present in Redis, routes reads directly to Primary DB (writer pool).
    pub async fn get_reader_pool_for_user(&self, user_id: Option<&str>) -> &PgPool {
        // 1. Global Circuit Breaker check
        if self.is_replica_lagging.load(std::sync::atomic::Ordering::Relaxed) {
            return &self.db.writer;
        }

        // 2. User Sticky Session check
        if let Some(uid) = user_id {
            if let Ok(mut conn) = self.redis.get().await {
                let key = format!("sticky_primary:{}", uid);
                let exists: bool = conn.exists(&key).await.unwrap_or(false);
                if exists {
                    tracing::debug!(user_id = %uid, "STICKY SESSION ACTIVE: Routing read directly to Primary DB");
                    return &self.db.writer;
                }
            }
        }

        &self.db.reader
    }

    /// Mark a user as sticky to Primary DB for `ttl_secs` after performing a write mutation.
    pub async fn set_user_sticky_primary(&self, user_id: &str, ttl_secs: u64) {
        if let Ok(mut conn) = self.redis.get().await {
            let key = format!("sticky_primary:{}", user_id);
            let _: Result<(), _> = conn.set_ex(&key, "1", ttl_secs).await;
            tracing::debug!(user_id = %user_id, ttl = ttl_secs, "Marked user sticky to Primary DB");
        }
    }
}

/// Build the complete application router with all routes registered.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health
        .route("/health", get(health::health_check))
        .route("/health/db", get(health::db_health_check))
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
        // Queue Stats
        .route("/queue/stats", get(queue_stats::get_stats))
        // Attach shared state
        .with_state(state)
}


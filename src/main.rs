// src/main.rs
//
// The application entry point. Its only responsibilities:
//   1. Load configuration
//   2. Initialize observability (tracing)
//   3. Connect to infrastructure (database pool)
//   4. Build the HTTP router
//   5. Start listening for connections
//
// RUST CONCEPT — #[tokio::main]:
//   This macro transforms `async fn main()` into a regular `fn main()` that
//   creates a Tokio multi-threaded runtime and runs the async function inside it.
//   The runtime manages a thread pool (default: one thread per CPU core).
//   Each thread can run thousands of async tasks concurrently via cooperative scheduling.
//
// WHAT HAPPENS AT THE OS LEVEL WHEN A REQUEST ARRIVES:
//
//   1. Linux kernel receives TCP segment on port 8080
//   2. Tokio's epoll watcher wakes up the listening task
//   3. Axum accepts the connection, spawns a new async task
//   4. Task executes the handler coroutine:
//      - If it hits await (e.g. sqlx query), Tokio parks it and runs other tasks
//      - When the DB responds, epoll wakes the task and it resumes
//   5. Response is written back to the TCP socket
//
// This is why Rust/Tokio can handle 10,000 concurrent requests on 8 OS threads —
// unlike Node.js (single-threaded) or Java with thread-per-request (10k threads = too much RAM).

mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;

use routes::AppState;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // ─── Step 1: Load .env file (only in development) ───────────────────────────
    // dotenvy reads .env and sets environment variables.
    // In production, variables are set by the deployment system (Docker, K8s secrets).
    dotenvy::dotenv().ok();

    // ─── Step 2: Initialize structured logging ───────────────────────────────────
    // tracing-subscriber reads RUST_LOG env var to set log level.
    // Example: RUST_LOG=info → logs info, warn, error
    //          RUST_LOG=debug → verbose, including sqlx query logs
    //          RUST_LOG=ecommerce_lab=debug,sqlx=warn → fine-grained control
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ecommerce_lab=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Ecommerce Lab V1");

    // ─── Step 3: Load application config ────────────────────────────────────────
    let config = config::AppConfig::from_env();
    tracing::info!(port = config.app_port, env = %config.app_env, "Configuration loaded");

    // ─── Step 4: Create database connection pool ──────────────────────────────────
    // This is async because it establishes real TCP connections to PostgreSQL.
    // If this panics, check: is Docker running? Is DATABASE_URL correct?
    let pool = db::create_pool(&config).await;
    tracing::info!("Database connection pool established (max_connections=10)");

    // ─── Step 5: Run pending migrations ─────────────────────────────────────────
    // sqlx::migrate! embeds all files from the `migrations/` directory at compile time.
    // This ensures the DB schema is always in sync when the app starts.
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Database migration failed");
    tracing::info!("Database migrations applied successfully");

    // ─── Step 6: Build the application router ───────────────────────────────────
    let state = AppState { db: pool };
    let app = routes::create_router(state)
        // TraceLayer logs every request: method, path, status, latency
        // This is your first observability layer — you'll rely on it heavily
        .layer(TraceLayer::new_for_http());

    // ─── Step 7: Bind TCP listener and serve ─────────────────────────────────────
    let addr = format!("0.0.0.0:{}", config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener — is the port already in use?");

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Health check: http://{}/health", addr);
    tracing::info!("Products API: http://{}/products", addr);

    // axum::serve hands connections to Tokio tasks indefinitely
    // This future only resolves if the server encounters a fatal error
    axum::serve(listener, app)
        .await
        .expect("Server encountered a fatal error");
}

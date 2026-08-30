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
mod events;
mod handlers;
mod jobs;
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
    dotenvy::dotenv().ok();

    // ─── Step 2: Initialize structured logging ───────────────────────────────────
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ecommerce_lab=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Ecommerce Lab V6 (Transactional Outbox Pattern & Kafka Event Streaming)");

    // ─── Step 3: Load application config ────────────────────────────────────────
    let config = config::AppConfig::from_env();
    tracing::info!(
        port = config.app_port,
        env = %config.app_env,
        primary_db = %config.database_url,
        replica_db = %config.read_database_url,
        kafka_brokers = %config.kafka_brokers,
        "Configuration loaded"
    );

    // ─── Step 4: Create primary & replica database pools & Redis pool ─────────
    let pools = db::create_pools(&config).await;
    tracing::info!("Primary and Replica PostgreSQL connection pools established (CQRS Enabled)");

    let redis_pool = db::create_redis_pool(&config);
    tracing::info!("Redis connection pool established for V2 caching & V4 job queue");

    // ─── Step 4b: Milestone V5 Replication Lag Circuit Breaker Initializer ────
    let is_replica_lagging = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let writer_pool_clone = pools.writer.clone();
    let is_lagging_flag = is_replica_lagging.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            if let Some(lag) = db::check_replication_lag(&writer_pool_clone).await {
                if lag > 500.0 {
                    if !is_lagging_flag.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        tracing::warn!(
                            "⚠️ REPLICA LAGGING ({} ms > 500ms SLA)! Tripping Circuit Breaker -> Routing 100% of reads to Primary",
                            lag
                        );
                    }
                } else if lag < 100.0 {
                    if is_lagging_flag.swap(false, std::sync::atomic::Ordering::Relaxed) {
                        tracing::info!(
                            "✅ REPLICA RECOVERED ({} ms)! Closing Circuit Breaker -> Resuming Replica reads",
                            lag
                        );
                    }
                }
            }
        }
    });
    tracing::info!("Milestone V5: Replication lag circuit breaker worker spawned (Threshold: 500ms)");

    // ─── Step 4c: Milestone V4 Background Worker Initialization ──────────────────
    jobs::worker::start_worker_pool(redis_pool.clone(), 5);
    tracing::info!("Milestone V4: Background job queue workers initialized (5 workers per instance)");

    // ─── Step 4d: Milestone V6 Transactional Outbox Relay & Kafka Consumers ────
    let kafka_producer = events::producer::KafkaProducer::new(config.kafka_brokers.clone());
    events::outbox_relay::start_outbox_relay(pools.writer.clone(), kafka_producer);
    tracing::info!("Milestone V6: Transactional Outbox Relay Worker spawned");

    events::consumers::start_notification_consumer(config.kafka_brokers.clone());
    events::consumers::start_analytics_consumer(config.kafka_brokers.clone());
    tracing::info!("Milestone V6: Kafka Consumer Groups ('notification-service-group', 'analytics-service-group') initialized");


    // ─── Step 5: Run pending migrations on Primary DB ───────────────────────────
    match sqlx::migrate!("./migrations").run(&pools.writer).await {
        Ok(_) => tracing::info!("Database migrations applied successfully to Primary DB"),
        Err(e) => tracing::warn!("Database migration skipped or notice: {e}"),
    }

    // ─── Step 6: Build the application router ───────────────────────────────────
    let state = AppState {
        db: pools,
        redis: redis_pool,
        config: config.clone(),
        is_replica_lagging,
    };
    let app = routes::create_router(state)
        .layer(TraceLayer::new_for_http());

    // ─── Step 7: Bind TCP listener and serve ─────────────────────────────────────
    let addr = format!("0.0.0.0:{}", config.app_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener — is the port already in use?");

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Health check: http://{}/health | http://{}/health/db", addr, addr);
    tracing::info!("Auth API: http://{}/auth/register | http://{}/auth/login", addr, addr);
    tracing::info!("Products API: http://{}/products", addr);
    tracing::info!("Orders API: http://{}/orders", addr);

    axum::serve(listener, app)
        .await
        .expect("Server encountered a fatal error");
}

# Post 019: Tracing Spans and Distributed Context Propagation in Async Rust
* **Target Audience**: DevOps Engineers, SRE Leads.
* **Viral Hook**: "How to trace a request ID as it moves through Nginx $\rightarrow$ Axum API $\rightarrow$ Postgres Query $\rightarrow$ Kafka Event using `tracing-opentelemetry`."
* **Core Code**:
  ```rust
  #[tracing::instrument(skip(db), fields(request_id = %req_id))]
  pub async fn process_order(req_id: &str, db: &PgPool) -> Result<Order> {
      tracing::info!("Starting order processing transaction");
      // Instrument nested sub-spans automatically
  }
  ```
* **Metrics Impact**: Reduced end-to-end distributed system request debugging time from hours to **under 1 minute**.
* **Cold Outreach DM**: "Hey [Name], saw your post on distributed tracing observability. Using Rust `tracing::instrument` macros propagates request trace IDs across async boundary calls into PostgreSQL and Kafka seamlessly. Shared our OpenTelemetry setup here!"

---

# Post 011: Tokio Select Macro — Managing Timeouts and Cancellations Cleanly in Rust APIs
* **Target Audience**: CTOs, Senior Backend Engineers.
* **Viral Hook**: "How to handle API request timeouts without leaving orphan database transactions running in the background. The `tokio::select!` cancellation safety rules."
* **Core Problem**: When an HTTP request client times out or cancels, un-canceled async tasks continue consuming CPU and DB connections on the server.
* **Technical Code**:
  ```rust
  tokio::select! {
      res = db_query_future => { res? }
      _ = tokio::time::sleep(Duration::from_secs(3)) => {
          Err(AppError::Timeout)
      }
  }
  ```
* **Metrics Impact**: Prevented orphan database query leaks; server connection cleanup latency reduced to **0ms** upon client disconnect.
* **Cold Outreach DM**: "Hey [Name], saw your update on API timeout handling. Using `tokio::select!` with cancellation-safe futures guarantees that client HTTP drops immediately terminate downstream DB operations, saving server resources. Shared our cancellation safety guide here!"

---

# Post 010: CPU-Bound Tasks vs Async Reactors — Why Image Resizing Crashed Our Axum Server
* **Target Audience**: Principal Engineers, Backend Architects.
* **Viral Hook**: "Processing a CPU-bound bcrypt or image resize operation inside an async function steals time from thousands of waiting network sockets. How `spawn_blocking` saves Tokio."
* **Core Problem**: Heavy CPU computation in async functions prevents the Tokio reactor from polling network sockets, causing incoming TCP connections to time out.
* **Technical Code**:
  ```rust
  // Offload heavy CPU bcrypt hashing away from Tokio reactor thread
  let password_hash = tokio::task::spawn_blocking(move || {
      bcrypt::hash(password, 10)
  }).await??;
  ```
* **Metrics Impact**: Eliminated socket timeouts; auth endpoint throughput increased by **340%** under 500 concurrent user registrations.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is scaling user registration & media processing services. Offloading CPU-heavy tasks like bcrypt hashing to Tokio's `spawn_blocking` pool prevents freezing async network reactors during peak signups. Documented our workload isolation pattern here!"

---

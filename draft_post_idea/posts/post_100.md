# Post 100: Redis Pipeline Batching — Executing 100 Redis Commands in 1 Network Roundtrip
* **Target Audience**: Backend Developers, Performance Engineers.
* **Viral Hook**: "Executing 100 `GET` requests sequentially takes 100ms. Executing them in a Redis Pipeline takes 1.2ms."
* **Core Problem**: Individual network roundtrips (`PING-PONG`) to Redis dominate execution time when processing bulk items.
* **Technical Code**:
  ```rust
  let mut pipe = redis::pipe();
  for id in product_ids {
      pipe.get(format!("product:{}", id));
  }
  let results: Vec<String> = pipe.query_async(&mut conn).await?;
  ```
* **Metrics Impact**: Bulk catalog lookup latency reduced from 110ms to **1.2ms (91x speedup)** under 2,000 VUs.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is optimizing Redis data access. Batching Redis commands using `redis::pipe()` reduces network roundtrips and cuts bulk retrieval latency from 100ms to 1ms. Shared our Rust pipeline code snippet here!"

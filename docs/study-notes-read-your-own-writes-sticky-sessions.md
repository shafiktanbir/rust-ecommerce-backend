# Study Notes: Read-Your-Own-Writes Consistency & Redis-Backed Sticky Sessions

> **Workspace**: `rust ecommerse-loop`  
> **Topic**: Distributed Systems Eventual Consistency, CQRS Race Conditions, and Redis-Backed Sticky Session Routing in Rust & PostgreSQL.

---

## 1. The Problem: Read-After-Write Consistency (Stale Read Bug)

In a CQRS Read-Write Splitting architecture:
* **Writes** (`POST`, `PUT`, `DELETE`) go to the **Primary Database**.
* **Reads** (`GET`) go to the **Read Replica Database**.

Because PostgreSQL streaming replication across TCP is **asynchronous**, a brief replication lag (e.g. 10ms–50ms) exists between Primary and Replica.

### ⚠️ The Bug Scenario
If a user creates an order or updates their profile and the frontend immediately redirects them to view their data:
1. `POST /orders` writes to **Primary DB**.
2. Frontend immediately calls `GET /orders`.
3. Server routes `GET /orders` to **Read Replica DB**.
4. **Replica has not received the WAL log yet**!
5. User gets an empty array (`[]`) or `404 Not Found`, causing user confusion and support tickets.

---

## 2. Architectural Comparison: Without vs. With Sticky Sessions

### ❌ Scenario A: Without Sticky Sessions (Stale Read Race Condition)

```text
Timeline WITHOUT Sticky Sessions:
─────────────────────────────────────────────────────────────────────────────────────
t = 0ms    User clicks "Buy Now" ──────► POST /orders
t = 2ms    Rust writes order to PRIMARY DB (Order #101 created!)
t = 5ms    Primary starts streaming WAL logs over network to REPLICA DB (Replication Lag = 30ms)
t = 10ms   User browser redirects ─────► GET /orders
t = 12ms   Rust receives GET /orders ──► Routes request to REPLICA DB
t = 13ms   REPLICA DB checks tables ───► Replica hasn't received WAL log yet!
                                         Returns: [] (Empty array!)
t = 15ms   User sees: "0 Orders Found!" ❌ (Stale Read Bug)
t = 35ms   Replica receives WAL log... (Too late!)
```

---

### ✅ Scenario B: With Redis-Backed Sticky Sessions (Guaranteed Consistency)

```text
Timeline WITH Redis-Backed Sticky Sessions:
─────────────────────────────────────────────────────────────────────────────────────
t = 0ms    User clicks "Buy Now" ──────► POST /orders
t = 2ms    Rust writes order to PRIMARY DB (Order #101 created!)
t = 3ms    💡 RUST SETS REDIS KEY:  `sticky_primary:{user_id}` (TTL = 5 seconds)
t = 5ms    Primary starts streaming WAL logs over network to REPLICA DB
t = 10ms   User browser redirects ─────► GET /orders
t = 12ms   Rust receives GET /orders ──► Checks Redis: Does `sticky_primary:{user_id}` exist?
t = 13ms   Redis says: YES! ───────────► 🛡️ RUST REROUTES READ DIRECTLY TO PRIMARY DB!
t = 15ms   PRIMARY DB checks tables ───► Returns: [Order #101] ✅
t = 16ms   User sees: "Order #101 Confirmed!" (Fresh Data Guaranteed!)
t = 5,000ms 5 seconds pass ────────────► Redis key expires. User reads return to Replica DB.
```

---

## 3. Rust & Axum Implementation Details

### 1. State Pool Switcher ([`src/routes/mod.rs`](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/src/routes/mod.rs))

```rust
impl AppState {
    /// Read-Your-Own-Writes Sticky Session Pool Switcher:
    /// Checks if a specific user/session has recently performed a write operation (within sticky TTL window).
    /// If sticky flag is present in Redis, routes reads directly to Primary DB (writer pool).
    pub async fn get_reader_pool_for_user(&self, user_id: Option<&str>) -> &PgPool {
        // 1. Global Circuit Breaker check (Lag > 500ms -> All reads to Primary)
        if self.is_replica_lagging.load(std::sync::atomic::Ordering::Relaxed) {
            return &self.db.writer;
        }

        // 2. Selective User Sticky Session check
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
```

### 2. Handler Integration ([`src/handlers/orders.rs`](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/src/handlers/orders.rs))

```rust
pub async fn create_order(...) -> AppResult<...> {
    let order = order_service::create_order(&state.db.writer, &state.redis, user_id, payload).await?;
    
    // Mark user sticky to Primary DB for 5 seconds after checkout
    state.set_user_sticky_primary(&claims.sub, 5).await;

    Ok((StatusCode::CREATED, Json(order)))
}

pub async fn list_orders(...) -> AppResult<...> {
    // Selects Primary DB if sticky key exists, otherwise Read Replica DB
    let pool = state.get_reader_pool_for_user(Some(&claims.sub)).await;
    let orders = order_service::list_user_orders(pool, user_id).await?;
    Ok(Json(orders))
}
```

---

## 4. Senior SRE Trade-offs & Engineering Analysis

| Aspect | Evaluation | Reason |
| :--- | :--- | :--- |
| **Consistency** | 🟢 **Strong (Per-User)** | The mutating user always receives strong read-after-write consistency. |
| **Primary Load** | 🟢 **Minimal Impact** | Only users who recently wrote data hit Primary DB for 5 seconds. All passive browsing traffic stays on Replica. |
| **Performance** | 🟢 **Sub-Millisecond Check** | Redis `EXISTS` lookup takes `< 1ms`, adding negligible overhead to the read handler. |
| **Fault Tolerance** | 🟢 **Graceful Degradation** | If Redis is down, `unwrap_or(false)` safely falls back to Replica DB without crashing. |

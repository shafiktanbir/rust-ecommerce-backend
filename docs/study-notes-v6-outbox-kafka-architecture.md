# Milestone V6 Study Notes: Transactional Outbox Pattern & Kafka Event Streaming

> **Architecture Roadmap**: Transitioning from V5 (CQRS Read/Write DB Replication + Redis Queue) to V6 (PostgreSQL Transactional Outbox Pattern + Apache Kafka Event Streaming).

---

## 1. Executive Summary & Core Problems Solved

While Milestone V5 solved database catalog read congestion via Primary/Replica CQRS, it left **3 enterprise reliability bottlenecks**:

1. **Dual-Write Inconsistency (Silent Data Loss)**: Writing to PostgreSQL (`COMMIT`) and pushing to Redis/Kafka (`LPUSH`/TCP) are two non-atomic calls. If network drops right after Postgres commits, the order is saved but background jobs are lost forever.
2. **Multi-Service Fan-Out Limits**: Redis queues are point-to-point (popped and deleted). Sending 1 order event to 5 microservices requires 5 duplicate queue writes, multiplying coupling and network load.
3. **Zero Event Replayability**: Once a transient queue message is processed, it is destroyed. If an downstream service crashes or suffers data corruption for 6 hours, historical events cannot be replayed.

**Milestone V6 eliminates all three using the PostgreSQL Transactional Outbox Pattern and Apache Kafka Event Streaming.**

---

## 2. Architecture & Design Strategies

### Strategy A: PostgreSQL Transactional Outbox Pattern

To prevent the Dual-Write problem, Axum **never makes an external network call to Kafka or Redis during the HTTP request**. Instead, it writes both the domain entity and the event to the same database inside a single ACID transaction.

#### Database Schema: `outbox` Table
```sql
CREATE TABLE outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type VARCHAR(64) NOT NULL, -- e.g. 'Order'
    aggregate_id UUID NOT NULL,          -- e.g. order_id
    event_type VARCHAR(64) NOT NULL,     -- e.g. 'OrderCreated'
    payload JSONB NOT NULL,              -- JSON payload of event
    status VARCHAR(32) NOT NULL DEFAULT 'pending', -- 'pending', 'processed', 'failed'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

CREATE INDEX idx_outbox_pending ON outbox(created_at) WHERE status = 'pending';
```

#### Atomic Outbox Transaction Flow
```text
[ Client (POST /orders) ]
          │
          ▼
 [ Axum Order Handler ]
          │
          ▼
┌──────────────────────────────────────────────────────────────────────┐
│ SINGLE ATOMIC POSTGRESQL TRANSACTION (Zero External Network Calls)   │
│                                                                      │
│  1. INSERT INTO orders (id, user_id, amount) VALUES (...)            │
│  2. INSERT INTO outbox (aggregate_type, aggregate_id, event_type,    │
│                         payload) VALUES ('Order', id, ...)           │
└──────────────────────────────────┬───────────────────────────────────┘
                                   │
                                   ▼
                       [ PostgreSQL Primary DB Disk ]
                                   │
                                   ▼
             (HTTP 201 Created Returned to User in < 15ms)
```

#### Outbox Relay Worker Mechanics
A background Rust process (or Debezium CDC engine) polls the `outbox` table and relays events to Kafka:

```text
               [ Outbox Relay Worker ]
                          │
                          ├─────────────────────────┐
                          │ 1. Read pending outbox  │
                          ▼                         │
             [ PostgreSQL outbox Table ]            │ 2. Send TCP to Kafka
                                                    ▼
                                       [ Kafka Topic: order-events ]
                                                    │
                                                    ▼
                                          (Kafka Responds ACK ✅)
                                                    │
                                                    ▼
                                          3. Mark status = 'processed'
```

- **Network Drop / Crash Safety**: If Kafka crashes or network drops during Step 2, the event remains safely stored on PostgreSQL disk inside the `outbox` table. The Relay retries sending until Kafka recovers (`At-Least-Once Delivery`).

---

### Strategy B: Kafka Event Streaming & Consumer Groups

Kafka acts as an **append-only distributed commit log on disk**.

```text
[ Axum Order Handler ] ──► (Outbox Relay) ──► [ Kafka Topic: order-events ]
                                                          │
       ┌────────────────────┬─────────────────────────────┼─────────────────────────────┐
       │ (Group A)          │ (Group B)                   │ (Group C)                   │ (Group D)
       ▼                    ▼                             ▼                             ▼
[ Email Service ]    [ Analytics DB ]             [ Fraud Engine ]             [ Inventory Warehouse ]
```

#### Key Components:
1. **Topic Partitioning Strategy**: Set `partition_key = customer_id`. All events for a specific customer land on the exact same log partition, guaranteeing that `OrderCreated` $\rightarrow$ `PaymentCompleted` $\rightarrow$ `OrderShipped` are processed in strict sequential order.
2. **Consumer Groups**: Independent microservice teams register distinct Consumer Groups. Each group maintains its own offset pointer in Kafka.
3. **Event Replayability**: If a bug corrupts the Analytics DB, the Analytics team resets their consumer group offset (`--to-offset 0`) and replays 7 days of historical events without affecting live production traffic.

---

## 3. Engineering Trade-offs & Architecture Decisions

| Trade-off Dimension | Choice Selected | Alternative Option | Engineering Rationale |
| :--- | :--- | :--- | :--- |
| **Outbox Storage** | **PostgreSQL `outbox` Table** | Direct Network Push to Kafka | Accepts minor DB write overhead (~1ms) to guarantee 100% zero data loss during network crashes. |
| **Message Broker** | **Apache Kafka / Redpanda** | RabbitMQ / Redis Queue | Prefers append-only disk log and native event replayability over RabbitMQ's transient message deletion. |
| **Delivery Semantics** | **At-Least-Once Delivery** | Exactly-Once Processing (2PC) | Avoids complex Two-Phase Commit performance penalties. Relies on **idempotent consumer tokens** in workers. |
| **Outbox Polling Engine** | **Rust `FOR UPDATE SKIP LOCKED`** | Debezium CDC (WAL Streaming) | Starts with simple, lightweight Rust SQL polling in V6; can seamlessly upgrade to Debezium CDC in V7+. |
| **Consistency Model** | **Eventual Consistency** | Synchronous RPC / 2PC | Offloads slow side-effects (emails, analytics, fraud) to background workers; returns HTTP 201 to users in < 15ms. |

---

## 4. Summary of System Evolution (V1 to V6)

```text
V1: Axum + PostgreSQL Monolith
  └─► Bottleneck: Low RPS, no caching, synchronous DB locks.

V2: Add Redis Cache-Aside + Auth JWT
  └─► Solved: Sub-millisecond catalog reads.

V3: Multi-Instance Cluster + Nginx Load Balancer (Hetzner VPC)
  └─► Solved: Scaled to 4,931 RPS @ 3,000 VUs.

V4: Async Background Job Queue (Redis List)
  └─► Solved: Decoupled slow email/invoice side-effects.

V5: PostgreSQL Streaming Replication + CQRS Dual Pools
  └─► Solved: Offloaded 90% read queries to Replica DB; added Lag Circuit Breaker.

V6: Transactional Outbox Pattern + Kafka Event Streaming
  └─► Solves: Dual-Write data loss, Multi-Service Fan-Out, and Event Replayability.
```

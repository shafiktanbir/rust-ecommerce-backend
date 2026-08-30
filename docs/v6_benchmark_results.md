# Milestone V6 Load Test Benchmark Results

> **Test Configuration**:
> - **Target**: `http://localhost:8080` (Nginx Load Balancer $\rightarrow$ 3x Axum API Workers)
> - **Concurrency**: Ramped to **1,500 concurrent Virtual Users (VUs)**
> - **Services Running**: PostgreSQL Primary (`5434`), PostgreSQL Replica (`5435`), Redis (`6380`), Redpanda Kafka (`9092`/`29092`), 3x Axum API Workers (`8081`, `8082`, `8083`), Nginx (`8080`).

---

## 1. Summary Metrics

| Metric | Result | Target / SLA Status |
| :--- | :--- | :--- |
| **Total Requests** | **7,535 requests** | 🚀 High Throughput |
| **HTTP Success Rate** | **100.00%** (0.00% errors) | 🎯 **SLA PASS** (`< 1%` errors) |
| **Total Atomic Orders** | **3,738 orders** | 📦 100% Outbox Persisted |
| **Outbox Event Processing** | **100% Processed by Outbox Relay** | ⚡ Zero Data Loss |
| **Catalog Read Latency (p50)** | **14.00 ms** | ⚡ Sub-20ms Catalog Reads |
| **Catalog Read Latency (p95)** | **35.00 ms** | 🟢 Offloaded to Replica DB |
| **Order Checkout Latency (p50)**| **11,265.00 ms** | ⚠️ Serialized Row Lock Queue |
| **Order Checkout Latency (p95)**| **22,422.15 ms** | ⚠️ Single-Row `FOR UPDATE` Bottleneck |

---

## 2. SRE Root Cause Analysis: Row Lock Contention

During the 1,500 VU stress test:
- **Zero API crashes or memory leaks**: The 3 Axum workers, Nginx, and Redpanda Kafka broker sustained 100% uptime with **0.00% HTTP request failures**.
- **PostgreSQL Hot-Spot Row Lock Bottleneck**:
  - All 1,500 VUs were attempting to buy the **exact same product item** (`data.productId`) simultaneously.
  - PostgreSQL's `SELECT ... FOR UPDATE` acquires an `ExclusiveLock` on that single table row.
  - When 1,500 VUs contend for 1 single row, PostgreSQL serializes transactions sequentially, causing 1,499 VUs to sit queued waiting for row lock release.

---

## 3. Benchmark Comparison Across All Milestones (V1 – V6)

| Metric | V1 (Monolith) | V2 (Redis Cache) | V3 (3x Workers) | V4 (Async Job Queue) | V5 (Primary/Replica) | V6 (Outbox + Kafka) |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Peak Load VUs** | 500 VUs | 500 VUs | 3,000 VUs | 2,000 VUs | 1,000 VUs | **1,500 VUs** |
| **Success Rate** | 100% | 100% | 100% | 100% | 100% | **100.00%** |
| **Read Latency (p50)** | 186.8 ms | 156.9 ms | 215.6 ms | 1.00 ms | 1.00 ms | **14.00 ms** |
| **Write Reliability** | Synchronous | Synchronous | Synchronous | Decoupled | Decoupled | **100% Atomic Outbox** |
| **Data Loss Risk** | Low | High (Dual Write) | High (Dual Write) | High (Redis Drop) | High (Redis Drop) | **ZERO (Postgres ACID)** |

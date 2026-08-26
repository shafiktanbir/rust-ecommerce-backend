# Milestone V4 Benchmark Results — Asynchronous Background Job Queue

## Executive Summary
Milestone V4 introduced a **Redis-backed Asynchronous Background Job Queue** (`deadpool-redis` + Tokio async worker pool) to decouple heavy post-checkout side effects (email confirmations, invoice PDF generation, analytics logging) from the HTTP `POST /orders` request-response lifecycle.

---

## Benchmark Setup & Load Configuration

* **Infrastructure**: 3x Axum API Worker containers + Nginx Load Balancer + PostgreSQL 15 + Redis 7
* **Load Test Script**: [`load-tests/v4_realistic_mixed_workload.js`](../load-tests/v4_realistic_mixed_workload.js)
* **Concurrency**: 2,000 Peak Virtual Users (VUs)
* **Workload Mix**:
  - **70% Catalog Reads** (`GET /products?limit=20&offset=X`)
  - **15% Product Detail Lookups** (`GET /products/:id`)
  - **15% Order Checkouts** (Authenticated `POST /orders` with JWT + DB Transaction + Redis Job Push)
* **User Behavior**: 1.0s – 2.5s randomized think time between requests

---

## Performance Metrics Summary

| Metric | Measured Benchmark Value | SLA Threshold | Status |
| :--- | :--- | :--- | :--- |
| **Total Requests Processed** | **58,844 requests** | N/A | 🚀 High Throughput |
| **Success Rate** | **100.00%** (58,844 / 58,844) | > 99.00% | 🎯 Zero Errors |
| **Overall HTTP Latency (p95)** | **28.38 ms** | < 500.00 ms | 🟢 **-94.3% Faster than SLA** |
| **Catalog Read Latency (p95)** | **21.00 ms** | < 200.00 ms | ⚡ **-89.5% Faster than SLA** |
| **Order Write Latency (p95)** | **103.00 ms** | < 600.00 ms | 🚀 **-82.8% Faster than SLA** |
| **Median Order Write Latency (p50)** | **5.00 ms** | N/A | ⚡ Ultra-Fast Ingress |
| **Background Job Failures** | **0 failed jobs** | 0 failures | 🛡️ 100% Reliable |

---

## Architectural Comparison: V3 (Synchronous) vs V4 (Asynchronous)

| Dimension | Milestone V3 In-Band Sync | Milestone V4 Decoupled Queue | Improvement |
| :--- | :--- | :--- | :--- |
| **`POST /orders` p95 Latency** | 1,400.00 ms (SLA FAIL) | **103.00 ms (SLA PASS)** | ⚡ **-92.6% Latency Reduction** |
| **PostgreSQL Pool Saturation** | Pinning connections for ~680ms | Releasing connections in **< 15ms** | 🛡️ Eliminates DB Pool Starvation |
| **Fault Isolation** | Downstream failure crashes checkout | Downstream failure retries in background | 🔒 100% Checkout Availability |

# Database Scaling Benchmarks & Capacity Formulas Guide

This document outlines industry market norms, capacity planning formulas (Little's Law, Connection Pool sizing, VU-to-RPS math), and empirical evidence gathered from load testing the Rust E-Commerce Lab.

---

## 1. Industry Market Scaling Tiers

In production systems, database scaling milestones are triggered by **Requests Per Second (RPS)**, **Query Latency ($W$)**, and **Row Lock Contention**, rather than raw registered user counts.

| Tier | Throughput Threshold | Architecture Pattern | Database Scaling Technique | Industry Benchmark Norm |
| :--- | :--- | :--- | :--- | :--- |
| **Tier 1: Monolith** | **< 100 RPS** (< 500 VUs) | Single PostgreSQL DB | Basic Indexes + App Connection Pool (10–20 connections) | Single DB node (e.g., AWS RDS `db.t4g.small` or Hetzner 2-core VPS). |
| **Tier 2: Production App** | **100 – 1,000 RPS** (500 – 5,000 VUs) | Redis Cache + PgBouncer | Redis Cache-Aside + DB Connection Pooling | **80%–90% of reads offloaded to Redis**. **PgBouncer** multiplexes 5,000 app connections into 30 DB connections. |
| **Tier 3: Scale E-Commerce** | **1,000 – 10,000 RPS** (5k – 50k VUs) | Primary-Replica Cluster | **Read Replicas (1 Primary Write, 2+ Read Copies)** | All `INSERT/UPDATE/DELETE` go to Primary DB. All `SELECT` queries go to Read Replicas via load balancer. |
| **Tier 4: Event-Driven** | **10,000 – 50,000 RPS** (50k – 500k VUs) | Queue Decoupling + Table Partitioning | PostgreSQL Partitioning + Kafka / Redis Queue | Large tables (`orders`) partitioned by date. Heavy write side-effects pushed to async job queues. |
| **Tier 5: Hyperscale** | **> 50,000 RPS** (> 500k VUs) | Horizontal DB Sharding | Distributed SQL (Citus, CockroachDB, Vitess) | Sharding the database by `tenant_id` or `user_id` across multiple physical primary databases. |

---

## 2. Core Capacity Planning Formulas

### A. Little’s Law (Database Queueing Math)

Little’s Law governs concurrency in queueing systems:

$$L = \lambda \cdot W$$

Where:
- $L$ = Number of active concurrent queries executing inside the database
- $\lambda$ = Arrival rate in Requests Per Second (RPS)
- $W$ = Average query latency in seconds

#### Scenario 1: Fast Indexed Queries ($10\text{ ms}$)
If your DB receives $\lambda = 1,000\text{ RPS}$ and average query latency is $W = 10\text{ ms} = 0.010\text{ s}$:
$$L = 1,000 \times 0.010 = 10\text{ active concurrent database queries}$$
*Result*: A small connection pool of 10 connections handles 1,000 RPS comfortably.

#### Scenario 2: Slow Unindexed Queries ($100\text{ ms}$)
If query latency degrades to $W = 100\text{ ms} = 0.10\text{ s}$:
$$L = 1,000 \times 0.10 = 100\text{ active concurrent database queries}$$
*Result*: A pool of 10 connections saturates immediately, forcing 90 incoming requests to queue in memory and spiking tail latency.

---

### B. PostgreSQL Connection Pool Sizing Formula

Setting `max_connections = 500` inside PostgreSQL degrades performance because PostgreSQL uses **1 OS process per connection**, causing heavy CPU context-switching thrashing.

The official PostgreSQL Core Team formula for optimal connection pool size:

$$\text{Optimal Pool Size} = (\text{CPU Cores} \times 2) + \text{Effective Spindle Count}$$

*(Note: For modern NVMe SSDs, Spindle Count is effectively 1 to 2).*

#### Calculation for a 4-Core Database Server:
$$\text{Optimal Pool Size} = (4 \times 2) + 1 = 9\text{ to }10\text{ connections}$$

- **Takeaway**: A pool of **10 to 20 connections per DB instance** yields the highest CPU cache locality and throughput.
- **To handle thousands of client connections**: Place **PgBouncer** in front of PostgreSQL to multiplex thousands of client connections down to 10–20 active DB connections.

---

### C. Concurrent Active Users (VUs) to Database RPS Formula

Users do not click continuously; they have **User Think Time** ($T_{\text{think}}$) while reading screens.

$$\text{Target RPS} = \frac{\text{Total Concurrent Active Users (VUs)}}{T_{\text{think}} + T_{\text{latency}}}$$

#### Example Calculation for 10,000 Active E-Commerce Users:
- Total Active Users = $10,000$
- Average Think Time between actions = $2.5\text{ seconds}$
- API Latency = $0.05\text{ seconds}$ ($50\text{ ms}$)

$$\text{Target RPS} = \frac{10,000}{2.5 + 0.05} = \frac{10,000}{2.55} \approx 3,921\text{ RPS}$$

#### Production Workload Distribution Math (80/20 Rule):
- **80% Reads ($3,136\text{ RPS}$)** $\rightarrow$ Handled by Redis Cache ($2,500\text{ RPS}$) + Read Replicas ($636\text{ RPS}$).
- **20% Writes ($785\text{ RPS}$)** $\rightarrow$ Handled by Primary DB + Async Background Queues.

---

## 3. Empirical V4 Benchmark Case Study

Telemetry gathered during V4 load testing on local 3-worker cluster (`v4_realistic_mixed_workload.js` & `v4_order_choke_test.js`):

### Empirical Fact 1: Read vs. Write Latency Divergence
```
Catalog Reads (Redis Cache):    p50 = 1.00 ms  |  p95 = 2.00 ms   ⚡ Sub-2ms
Order Writes (PostgreSQL DB):   p50 = 258 ms   |  p95 = 636 ms    ⚠️ 1.96s Max Spike
```
*Diagnosis*: Redis caching offloads read traffic effectively (2 ms), but order writes bound to PostgreSQL experience latency spikes up to 1.96s under lock contention.

### Empirical Fact 2: Connection Pool Queueing
Configured in `src/db/mod.rs`: `PgPoolOptions::new().max_connections(10)`.
- 3 Axum API workers $\times$ 10 connections = 30 DB connections.
- Under 500 concurrent checkout VUs, 470 requests wait in application memory for an available pool connection, driving tail latency to 636 ms (p95).

### Empirical Fact 3: Database Row Lock Contention
In `src/repositories/product_repository.rs`:
```sql
UPDATE products
SET inventory_count = inventory_count - 1, updated_at = NOW()
WHERE id = $1 AND inventory_count > 0
```
- Under 500 VUs targeting 5 items, PostgreSQL applies an `ExclusiveLock` per product row.
- Transactions execute serially per product row, queueing requests at the database engine level.

---

## 4. Next Milestone (V5) Architectural Fixes

| Bottleneck Identified | Empirical Metric | Milestone V5 Solution |
| :--- | :--- | :--- |
| **Connection & CPU Contention** | Reads & Writes compete for 30 pool connections | **Read Replicas**: Offload 100% of read queries to secondary DB replicas; Primary DB dedicated 100% to Writes. |
| **Lock Contention Spikes** | 1.96s max spike under 500 VU choke | **Optimized Indexing & Isolation**: Reduce transaction lock holding times. |
| **Slow Query Scans** | Full table scans as `orders` table grows | **Composite B-Tree Indexes**: Replace $O(N)$ sequential scans with $O(\log N)$ microsecond B-Tree lookups. |

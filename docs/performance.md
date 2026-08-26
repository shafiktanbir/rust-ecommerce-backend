# Performance Engineering Guide

## The Core Principle

> "The system is slow because of X."

Never guess. Always measure. This document explains the metrics we track and how to use them
to identify bottlenecks with evidence.

---

## Metric Definitions

### Throughput

| Metric | What it measures | How to read it |
|--------|-----------------|----------------|
| **RPS** (Requests Per Second) | How many requests the system processes per second | Higher is better. Dropping RPS under stable load = the system is rejecting or timing out |

### Latency

| Metric | What it measures | Why it matters |
|--------|-----------------|----------------|
| **p50** | 50th percentile — the median response time | Half of users experience ≤ this latency |
| **p95** | 95th percentile | 1 in 20 requests takes this long. This is what "most users" experience at peak |
| **p99** | 99th percentile | The slowest 1% — often reveals hidden queuing or lock contention |
| **p99.9** | 99.9th percentile | "Tail latency" — affects 1 in 1000 requests. Critical at 10k RPS (= 10 bad requests/sec) |

**Why p95/p99 matter more than p50:**

If p50 = 10ms but p99 = 8000ms, you have a serious problem affecting 1% of users.
With 10,000 users/hour, that's 100 users experiencing 8-second waits.

### Error Rate

```
Error Rate = (Failed Requests / Total Requests) × 100%

Acceptable:
  < 0.1% for most APIs
  0% for financial transactions (orders, payments)

Alert threshold:
  > 1% = something is wrong
  > 5% = service is degraded
  > 10% = service is effectively down
```

### System Resources

| Metric | Alert Threshold | What it means |
|--------|----------------|---------------|
| **API CPU** | > 80% sustained | App is CPU-bound — profile or add instances |
| **API Memory** | Steadily growing | Memory leak — profile heap |
| **DB CPU** | > 70% sustained | DB is bottleneck — check slow queries |
| **DB Connections** | > 80% of max_connections | Pool exhaustion imminent |
| **DB Query Latency** | p95 > 50ms | Indexes missing or queries inefficient |
| **Lock Contention** | Waiting events in pg_stat_activity | Write conflicts — affects flash sale |

---

## How to Identify a Bottleneck

### Step 1: Run a load test (k6)

```bash
k6 run --vus 100 --duration 60s load-tests/products.js
```

### Step 2: Look at the shape of the degradation

```
Scenario A: RPS plateaus but latency stays low
  → The system hit its throughput ceiling but is handling it gracefully
  → Look at: CPU cores, connection pool size

Scenario B: Latency rises sharply while RPS stays constant
  → Requests are queueing somewhere
  → Look at: DB connection pool, DB lock contention, query time

Scenario C: Error rate increases
  → The system is actively rejecting requests
  → Look at: timeout errors, connection refused, DB max_connections hit
```

### Step 3: Check database metrics

```sql
-- Which queries are slowest right now?
SELECT query, calls, total_exec_time, mean_exec_time, rows
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 20;

-- Are connections being held too long?
SELECT pid, state, wait_event_type, wait_event, query, query_start
FROM pg_stat_activity
WHERE datname = 'ecommerce_lab'
ORDER BY query_start;

-- Are any rows being locked?
SELECT blocked_locks.pid AS blocked_pid,
       blocked_activity.query AS blocked_query,
       blocking_locks.pid AS blocking_pid,
       blocking_activity.query AS blocking_query
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
JOIN pg_catalog.pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.granted;
```

### Step 4: Use EXPLAIN ANALYZE on slow queries

```sql
-- Are we doing sequential scans on large tables?
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM products ORDER BY created_at DESC LIMIT 20;
```

Look for:
- `Seq Scan` on large tables → missing index
- High `rows removed by filter` → predicate needs an index
- `Rows Fetched: 1` after scanning 10,000 → wrong index or no index

---

## Load Test Recording Template

Use this template to record results from every k6 run.
Store results in `load-tests/results/YYYY-MM-DD-vN.md`.

```
Date: YYYY-MM-DD
Version: V1
Endpoint tested: GET /products

Load profile:
  VUs: 100
  Duration: 60s
  Ramp-up: 10s

Results:
  RPS:         ___
  p50:         ___ ms
  p95:         ___ ms
  p99:         ___ ms
  Error rate:  ___ %

System state at peak:
  API CPU:            ___ %
  API Memory:         ___ MB
  DB CPU:             ___ %
  DB connections:     ___ / 100
  DB query latency p95: ___ ms

Observations:
  -

Suspected bottleneck:
  -

Next action:
  -
```

---

## Future Measurement Tools

| Tool | Purpose | When We Add It |
|------|---------|----------------|
| **k6** | HTTP load testing | V1 |
| **pg_stat_statements** | Slow query analysis | V1 (already in PostgreSQL) |
| **Prometheus + Grafana** | Real-time metrics dashboard | V8 |
| **tokio-console** | Async task profiling | V3 (if CPU-bound) |
| **flamegraph** | CPU profiling | V3 |
| **pgBadger** | PostgreSQL log analysis | V5 |

---

## Connection Pool Sizing Formula (Little's Law)

To calculate how many database connections your system needs under peak load:

```text
Connections Needed = Requests Per Second * Average Query Duration (in seconds)
```

### Queue Delay Calculation:
```text
Queue Wait Time = Waiting Requests / Throughput
```

#### Example (500 VUs Stress Test):
- `Throughput`: 1,800 req/sec
- `Average Query Duration`: 0.015 seconds
- `Connections Needed`: `1,800 * 0.015 = 27 connections`
- `Pool Limit`: 10 connections
- `Result`: Pool saturation (100%), 490 requests wait in line, causing `p95` latency to jump to **574ms**.

---

## Official PostgreSQL Connection Pool Formula

```text
Optimal Pool Size = (CPU Cores * 2) + Disk Count
```

### Why increasing pool size to 50 made performance WORSE (786 RPS, 1.31s latency):

1. **Process-Per-Connection**: PostgreSQL creates a separate OS process for every connection. 50 connections = 50 heavy processes competing for CPU cores.
2. **CPU Context Switching**: The CPU spends more time switching between 50 processes than executing SQL queries.
3. **Disk I/O Bottleneck**: 50 processes reading/writing to 1 hard drive causes severe disk queueing.

**Rule**: Never solve scaling by simply raising database pool connections. Offload read queries to an in-memory cache like Redis (V2).

---

## Benchmark Milestone History

| Milestone | Architecture Profile | Peak RPS | p50 Latency | p95 Latency | Error Rate | Date |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **V1 Baseline** | Single VPS (100 VUs) | 1,200 RPS | 45 ms | 120 ms | 0.00% | 2026-08-20 |
| **V2 Caching** | Localhost + Redis (500 VUs) | 3,500 RPS | 15 ms | 45 ms | 0.00% | 2026-08-22 |
| **V3 Hetzner Multi-Node** | 2 Workers + Nginx LB + VPC (3,000 VUs) | **4,665.84 RPS** | **215.69 ms** | **452.68 ms** | **0.00%** | **2026-08-25** |

---

## 💡 Converting RPS to Real-World User Scale (DAU, MAU & Investor Pitch)

### 🧮 Mathematical Formulas

To convert raw HTTP throughput (**RPS**) into business user metrics:

#### 1. Peak Concurrent Users (Flash Sale Scale)
Human users spend time reading pages or browsing images before clicking the next link (**Think Time $T$**, typically 5s to 10s):

$$\text{Peak Concurrent Active Users} = \text{RPS} \times \text{Think Time (seconds)}$$

* At $T = 5\text{s}$: $4,665.84 \times 5 = \mathbf{23,329 \text{ Simultaneous Active Users}}$
* At $T = 10\text{s}$: $4,665.84 \times 10 = \mathbf{46,658 \text{ Simultaneous Active Users}}$

#### 2. Daily Active Users (DAU)
Assuming an average active user makes 20 API requests during a daily session (login, search, product views, checkout):

$$\text{Daily Requests} = \text{RPS} \times 86,400 \text{ seconds/day} = 4,665.84 \times 86,400 = \mathbf{403,128,576 \text{ requests/day}}$$

$$\text{Daily Active Users (DAU)} = \frac{403,128,576 \text{ daily requests}}{20 \text{ requests/user}} = \mathbf{20.15 \text{ Million DAU}}$$

#### 3. Monthly Active Users (MAU)
Consumer platforms observe a DAU-to-MAU ratio of 15% to 20% ($\text{MAU} \approx \text{DAU} \times 5$):

$$\text{MAU Capacity} = 20.15 \text{ Million DAU} \times 5 = \mathbf{100.75 \text{ Million MAU}}$$

---

### 🎤 Investor Pitch Response Script

When an investor asks: *"How many users can this architecture handle?"*

> **CTO Answer**:
> *"On our baseline €20/month cloud infrastructure (4 small Hetzner VMs), our Rust backend processes **4,665 requests per second** with **100% zero-error reliability** and **215ms median latency**.
>
> In business metrics, this capacity supports:
> 1. **Peak Traffic**: **23,000 to 46,000 users simultaneously** in high-demand flash sales.
> 2. **Daily Active Scale**: Over **400 Million requests per day**, supporting **20 Million Daily Active Users (DAU)**.
> 3. **Total Platform Base**: Over **100 Million Monthly Active Users (MAU)**.
>
> Because our Axum micro-cluster architecture is horizontally scalable, doubling our capacity to 10,000+ RPS simply requires deploying additional worker instances with zero code changes."*





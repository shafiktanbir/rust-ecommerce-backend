# Milestone V3 — Hetzner VPS Load Test Results & Baseline Comparison

This document stores the empirical load test benchmark results executed on **August 25, 2026** against a 4-node Hetzner Cloud cluster (`cx23` VMs, Nuremberg `nbg1`).

---

## 📊 Benchmark Summary Table

| Metric | Result Value | SRE Evaluation |
| :--- | :--- | :--- |
| **Peak Throughput (RPS)** | **`4,665.84 req/sec`** | 🚀 High Throughput Baseline |
| **Total Requests Processed** | **`307,483 requests`** | ✅ 300,000+ Requests in 65s |
| **HTTP Error Rate** | **`0.00%` (0 / 307,483)** | 🎯 100% Zero-Error Reliability |
| **Peak Virtual Users (VUs)** | **`3,000 VUs`** | ⚡ Scale Verified at 3,000 Concurrency |
| **Median Latency (p50)** | **`215.69 ms`** | ⚡ Fast Concurrent Processing |
| **p90 Latency** | **`334.61 ms`** | 🟢 Smooth Tail Distribution |
| **p95 Latency** | **`452.68 ms`** | 🟡 Acceptable under 3,000 VU Heavy Load |
| **Data Transferred** | **`121 MB Received / 34 MB Sent`** | 🌐 Ingress/Egress Bandwidth Verified |

---

## ⚙️ Infrastructure Specifications

* **Cloud Provider**: Hetzner Cloud (Location: `nbg1` - Nuremberg, Germany).
* **Instance Type**: 4 x `cx23` (2 vCPU, 4GB RAM each).
* **Cluster Toplogy**:
  * `ecommerce-nginx-lb` (`10.0.1.10`, Public IPv4): Nginx 1.24 load balancer with `least_conn` and `keepalive 512`.
  * `ecommerce-api-1` (`10.0.1.11`, Private VPC): Axum Rust API worker.
  * `ecommerce-api-2` (`10.0.1.12`, Private VPC): Axum Rust API worker.
  * `ecommerce-db` (`10.0.1.20`, Private VPC): PostgreSQL 15-alpine & Redis 7-alpine container host.
* **Golden Image**: Pre-baked Packer Snapshot ID `424302356`.

---

## 📈 Detailed k6 Execution Log

```
  █ TOTAL RESULTS

    checks_total.......: 307482  4665.82/s
    checks_succeeded...: 100.00% 307482 out of 307482
    checks_failed......: 0.00%   0 out of 307482

    ✓ list status is 200
    ✓ single product status is 200

    HTTP
    http_req_duration..............: avg=252.84ms min=174.1ms med=215.69ms max=2.00s p(90)=334.61ms p(95)=452.68ms
    http_req_failed................: 0.00%  0 out of 307483
    http_reqs......................: 307483 4665.84/s

    EXECUTION
    iterations.....................: 153741 2332.91/s
    vus_max........................: 3000
```

---

## 🎯 Future Milestone Targets (Comparison Matrix)

| Milestone | Target Architecture | Target RPS | Target p95 Latency | Status |
| :--- | :--- | :--- | :--- | :--- |
| **V1 Baseline** | Single VPS Localhost | ~1,200 RPS | ~120 ms | Completed |
| **V2 Caching** | Localhost + Redis Cache | ~3,500 RPS | ~45 ms | Completed |
| **V3 Hetzner Multi-Node** | 2 Workers + Nginx LB + VPC | **`4,665 RPS`** | **`452 ms`** | **Completed (Current Baseline)** |
| **V4 Async DB Scaling** | Read Replicas / PgBouncer | Target: 8,000+ RPS | Target: < 150 ms | Planned |

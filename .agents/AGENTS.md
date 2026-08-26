# Agent Behavior Guidelines - Rust High-Throughput E-Commerce Lab

## sre-devops-performance-engineer-persona

Whenever operating in this repository (`rust ecommerse-loop`), all AI agents MUST adopt the persona and skills of a **Senior SRE (Site Reliability Engineer), Senior DevOps Architect, and Principal Performance Engineer**:

1. **SRE & DevOps Mindset**:
   - **Immutable Infrastructure**: Prefer pre-baked Golden Images (Packer) over dynamic, slow runtime scripts.
   - **Zero Idle Compute Cost**: Always teardown infrastructure after benchmark tests complete (`terraform destroy`).
   - **Empirical Diagnostics**: Inspect logs, TCP connection states, CPU utilization, and HTTP response codes before proposing fixes.
   - **Strict Operational Security**: Never hardcode secrets or Hetzner API tokens into code; always use environment variables (`HCLOUD_TOKEN`).

2. **Performance Engineering Discipline**:
   - **High-Concurrency OS Tuning**: Enforce Linux kernel sysctl optimization for high throughput load testing (`net.core.somaxconn=65535`, `net.ipv4.tcp_max_syn_backlog=65535`, `fs.file-max=2097152`).
   - **Load Testing Benchmarks**: Evaluate application response under 3,000+ Virtual Users (VU) using k6, monitoring p95/p99 latency, RPS, and 5xx error thresholds.
   - **Async Rust Performance**: Optimize Axum, Tokio runtime, SQLx connection pools, and Redis pipelines for maximum throughput and minimum memory overhead.

---

## hetzner-packer-benchmark-strategy-rules

Whenever running Hetzner Cloud infrastructure benchmarks in this workspace:

1. **Pre-baked Packer Golden Image Strategy**:
   - Always use Packer (`infrastructure/v3-nginx-cluster/packer/v3-golden-image.pkr.hcl`) to pre-bake Ubuntu 24.04 OS, Docker, Postgres 15 & Redis 7 container images, Nginx, kernel sysctl tuning (`net.core.somaxconn=65535`), and the compiled Rust binary into a Hetzner Cloud Snapshot (`v3-golden-image-v1`).
   - Avoid dynamic runtime provisioning with Ansible (`apt update`, SSH file copies) on every `terraform apply` spawn loop to keep VM boot times under **30 seconds**.

2. **Benchmark Pipeline Scripts**:
   - Master Pipeline: `./scripts/run_v3_full_packer_benchmark.sh` (Compiles Rust binary, builds Packer snapshot, spawns Terraform VPS, verifies HTTP health, runs k6 test, and auto-destroys).
   - Image Builder: `./scripts/build_v3_packer_image.sh`
   - Fast Benchmark Runner: `./scripts/run_v3_hetzner_benchmark.sh` (supports `SKIP_ANSIBLE=true`)

3. **Cost-Saving & Guaranteed Teardown Protocol**:
   - Hetzner Cloud snapshots are billed pro-rated hourly based on compressed storage size (~€0.0143/GB/month, i.e., ~€0.00005/hr for a 3GB image).
   - Temporary Packer build instances run for ~3 mins (~€0.0003 cost) and are destroyed immediately upon snapshot creation.
   - Always ensure auto-teardown of benchmark VPS instances (`terraform destroy -auto-approve`) post-k6 benchmark to maintain zero idle compute costs.

4. **HTTP Health Check Polling**:
   - Poll `http://$NGINX_IP:8080/health` directly with backoff retries rather than relying solely on SSH port loops.

---

## realistic-k6-user-benchmark-rules

Whenever designing, executing, or analyzing load test benchmarks in this repository:

1. **Mandatory Production Workload Simulation**:
   - All AI agents MUST construct k6 load tests that simulate realistic user activity mixes rather than 100% synthetic read loops.
   - **Standard Production Ratio**:
     - **70% Catalog Reads** (`GET /products?limit=20`) -> Cached reads
     - **15% Uncached Product Detail Lookups** (`GET /products/:id`) -> Cache miss / DB read
     - **15% Authenticated Order Checkouts** (`POST /orders`) -> Cryptographic JWT validation + PostgreSQL row lock & write transaction
2. **User Think Time Requirement**:
   - Enforce realistic user think time between actions (`sleep(1.0 + Math.random() * 1.5)` pause) to reflect real human browsing behavior instead of zero-delay bot spamming.
3. **Transparent Reporting**:
   - Whenever evaluating RPS, agents MUST explicitly distinguish between **Pure Read Synthetic Throughput** (e.g. 4,600+ RPS) and **Realistic Mixed Workload Throughput** (e.g. 1,500 - 2,200 RPS).


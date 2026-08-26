# Load Tests

This directory contains k6 load test scripts for each version of the system.

## Philosophy

> "Do not optimize without measurements."

We start with small user counts and increase gradually.
For every test we record: RPS, p50/p95/p99, error rate, CPU, memory, DB utilization.

## Test Progression

```
V1: Establish baseline
  10 VUs  → 1 min
  100 VUs → 2 min
  500 VUs → 2 min

V2+: Compare before/after after each architectural change
```

## Directory Structure (will grow with versions)

```
load-tests/
├── README.md             ← this file
├── results/              ← test result records (created when we run first test)
│   └── .gitkeep
└── (k6 scripts added in V2 when we have auth + order endpoints)
```

## Prerequisites

```bash
# Install k6
# Linux:
sudo gpg -k
sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
sudo apt-get update && sudo apt-get install k6
```

## Running Load Tests

```bash
# V3 Pure Read Benchmark (3,000 VUs)
TARGET_URL="http://<LB_IP>:8080" k6 run load-tests/v3_stress_3000vu.js

# V4 Realistic Mixed Workload Benchmark (70% Read / 15% Detail / 15% Authenticated Order Writes)
TARGET_URL="http://<LB_IP>:8080" k6 run load-tests/v4_realistic_mixed_workload.js
```

## Recording Results

After every load test, record results in `load-tests/results/` following the template in
[docs/performance.md](../docs/performance.md).

Never discard old results — before/after comparisons are the proof that architectural changes work.


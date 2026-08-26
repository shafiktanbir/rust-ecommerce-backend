# Ecommerce Lab — Rust Scaling Laboratory
# Version 1: Monolith

## What is this?

This is a **scaling laboratory** — not just an e-commerce clone.

The goal is to build, measure, break, and scale a real Rust application from a simple monolith
to a system capable of handling **10,000 concurrent users**.

Each version introduces exactly one new architectural component — only when measurements show it is necessary.

---

## V1 Architecture

```
Client (curl / k6 / browser)
         ↓
  Axum HTTP API (Rust)
         ↓
  PostgreSQL (Docker)
```

Simple. Intentionally limited. The first bottlenecks will be discovered through load testing.

---

## Technology

| Component | Technology | Why |
|-----------|-----------|-----|
| Language | Rust | Memory safety + zero-cost async, excellent for high concurrency |
| HTTP | Axum | Ergonomic, built on Hyper + Tower |
| Async | Tokio | Multi-threaded async runtime |
| Database | PostgreSQL 15 | Reliable, ACID, excellent at concurrent writes |
| Driver | SQLx | Compile-time verified SQL |
| Logging | tracing | Structured, async-aware |
| Local infra | Docker Compose | Reproducible local environment |

---

## Prerequisites

- Rust (stable) — install via [rustup.rs](https://rustup.rs)
- Docker + Docker Compose
- `sqlx-cli` for running migrations manually

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
```

---

## Running Locally

### 1. Copy environment variables
```bash
cp .env.example .env
```

### 2. Start PostgreSQL
```bash
docker compose up -d
```

### 3. Run the application (migrations run automatically on startup)
```bash
cargo run
```

The server starts at `http://localhost:8080`.

---

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/products` | List products (limit, offset) |
| GET | `/products/:id` | Get product by UUID |
| POST | `/products` | Create product |

### Example requests

```bash
# Health check
curl http://localhost:8080/health

# Create a product
curl -X POST http://localhost:8080/products \
  -H "Content-Type: application/json" \
  -d '{"name": "Mechanical Keyboard", "price": 129.99, "inventory_count": 100}'

# List products
curl "http://localhost:8080/products?limit=10&offset=0"

# Get product by ID (replace with actual UUID from create response)
curl http://localhost:8080/products/YOUR-UUID-HERE
```

---

## Project Structure

```
src/
├── main.rs             # Startup: config → pool → router → listen
├── config/             # Environment variable loading
├── db/                 # PostgreSQL & Redis connection pool setup
├── errors/             # Centralized error types → HTTP responses
├── models/             # Pure data structs (Product, User, Order)
├── repositories/       # All SQL queries & Redis cache repositories
├── services/           # Business logic layer
├── handlers/           # HTTP request/response boundary
├── routes/             # Router assembly + AppState
└── middleware/         # Auth JWT extractor & request tracing

migrations/             # SQL schema migrations (run automatically)
docker-compose.yml      # PostgreSQL & Redis local setup
docs/
├── architecture/       # V1 architecture explanation & concurrency specs
├── decisions/          # Architecture Decision Records (ADRs)
├── mistakes/           # SRE incident postmortems & mistake logs
├── scenarios/          # 4-Layer SRE diagnostic funnel & incident handbooks
└── performance.md      # Metrics reference, Little's Law & measurement guide
infrastructure/         # Terraform, Ansible & Packer cluster scripts
load-tests/             # k6 load test scripts
scripts/                # Automated Hetzner Cloud benchmark execution scripts
```

---

## Layer Responsibilities

```
HTTP Request
     ↓
Handler       — extract params, call service, return HTTP response
     ↓
Service       — business rules, validation, cache-aside logic
     ↓
Repository    — SQL queries & Redis cache commands only
     ↓
PostgreSQL / Redis
```

Each layer has a single responsibility. This makes the system easier to:
- Test (mock the repository in unit tests)
- Scale (add a cache between service and repository)
- Debug (SQL problems are always in repositories/)

---

## Version Roadmap

| Version | Component Added | Trigger | Status |
|---------|----------------|---------|--------|
| V1 | Rust API + PostgreSQL | Baseline | ✅ COMPLETE |
| V2 | Redis cache-aside layer | Cache-miss rate & read throughput | ✅ COMPLETE |
| V3 | Load balancer + 3x API instances (Hetzner Cloud VPC) | Single instance CPU saturated (4,931 RPS achieved) | ✅ COMPLETE |
| V4 | Background job queue | Synchronous operations too slow | ⬜ Planned |
| V5 | DB indexes + read replicas | DB CPU bottleneck | ⬜ Planned |
| V6 | Kafka event streaming | Need durable async processing | ⬜ Planned |
| V7 | Docker + Kubernetes | Need horizontal scaling | ⬜ Planned |
| V8 | Autoscaling + observability | Need production readiness | ⬜ Planned |
| V9 | Failure testing | Need resilience | ⬜ Planned |
| V10 | 10k user load test | Final validation | ⬜ Planned |

---

## Documentation Index

- 📚 [**Master Documentation Index**](docs/README.md)
- 📐 [**V1 Monolith Architecture**](docs/architecture/v1.md)
- 📊 [**Performance Metrics Guide**](docs/performance.md)
- 📘 [**Hetzner V3 Benchmark Mistakes & SRE Solutions**](docs/mistakes/hetzner_v3_benchmark_mistakes.md)
- 📖 [**502/504 Ingress Outage & 4-Layer Diagnostic Scenario**](docs/scenarios/sre_502_504_diagnostic_scenario.md)
- 📜 [**ADR-001: Why PostgreSQL Monolith**](docs/decisions/001-postgresql.md)


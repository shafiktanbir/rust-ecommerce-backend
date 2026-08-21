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
├── db/                 # PostgreSQL connection pool setup
├── errors/             # Centralized error types → HTTP responses
├── models/             # Pure data structs (Product, User)
├── repositories/       # All SQL queries live here
├── services/           # Business logic layer
├── handlers/           # HTTP request/response boundary
├── routes/             # Router assembly + AppState
└── middleware/         # Request tracing, future auth

migrations/             # SQL schema migrations (run automatically)
docker-compose.yml      # PostgreSQL local setup
docs/
├── architecture/v1.md  # V1 architecture explanation
├── performance.md      # Metrics reference and measurement guide
└── decisions/          # Architecture Decision Records (ADRs)
load-tests/             # k6 load test scripts (added in V2)
```

---

## Layer Responsibilities

```
HTTP Request
     ↓
Handler       — extract params, call service, return HTTP response
     ↓
Service       — business rules, validation, orchestration
     ↓
Repository    — SQL queries only
     ↓
PostgreSQL
```

Each layer has a single responsibility. This makes the system easier to:
- Test (mock the repository in unit tests)
- Scale (add a cache between service and repository)
- Debug (SQL problems are always in repositories/)

---

## Version Roadmap

| Version | Component Added | Trigger |
|---------|----------------|---------|
| V1 | Rust API + PostgreSQL | Baseline |
| V2 | Redis cache | Cache-miss rate too high |
| V3 | Load balancer + multiple instances | Single instance CPU saturated |
| V4 | Background job queue | Synchronous operations too slow |
| V5 | DB indexes + read replicas | DB CPU bottleneck |
| V6 | Kafka event streaming | Need durable async processing |
| V7 | Docker + Kubernetes | Need horizontal scaling |
| V8 | Autoscaling + observability | Need production readiness |
| V9 | Failure testing | Need resilience |
| V10 | 10k user load test | Final validation |

---

## Documentation

- [V1 Architecture](docs/architecture/v1.md)
- [Performance Metrics Guide](docs/performance.md)
- [ADR-001: Why PostgreSQL Monolith](docs/decisions/001-postgresql.md)

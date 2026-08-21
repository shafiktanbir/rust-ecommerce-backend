# Build a Scalable Rust E-commerce Learning Lab

You are my senior backend engineering mentor and coding assistant.

I am a beginner in large-scale backend/system design. I have backend development experience, but I have never personally built and operated a system at 10,000-user scale.

I want to learn large-scale backend engineering by actually building, measuring, breaking, and scaling a real application.

## Project

Create a Rust e-commerce application with a flash-sale feature.

The goal is NOT simply to build an e-commerce clone.

The goal is to create a **scaling laboratory** where I can gradually evolve the system from a simple monolith into a system capable of handling high concurrency.

The project should eventually target:

* 10,000 concurrent users
* high request throughput
* flash-sale traffic spikes
* safe inventory handling
* low API latency
* horizontal scaling
* graceful failure
* observability
* load testing

## VERY IMPORTANT LEARNING RULE

Do NOT build the final distributed architecture immediately.

I want to experience why each architectural component becomes necessary.

Therefore build the system in versions.

### Version 1

Start with only:

```text
Client
   ↓
Rust API
   ↓
PostgreSQL
```

Do NOT add Redis, Kafka, Kubernetes, microservices, or complex infrastructure yet.

The first version should be a clean modular monolith.

After V1 works, we will load-test it and identify bottlenecks.

Then we will introduce new components ONLY when there is an engineering reason for them.

Possible future evolution:

```text
V1
Rust API + PostgreSQL

        ↓

V2
Add Redis

        ↓

V3
Add load balancer + multiple API instances

        ↓

V4
Add background jobs / message queue

        ↓

V5
Database optimization + read replicas

        ↓

V6
Kafka/event-driven processing

        ↓

V7
Docker + Kubernetes

        ↓

V8
Autoscaling + observability

        ↓

V9
Failure testing + resilience

        ↓

V10
10k concurrent-user load testing
```

Do not skip these stages.

---

# Technology

Use:

* Rust
* Axum for HTTP API
* Tokio for async runtime
* SQLx for PostgreSQL
* PostgreSQL
* Docker Compose for local infrastructure
* serde for serialization
* tracing for application logging

Use a clean project structure.

Do not introduce unnecessary frameworks or abstractions.

---

# Initial Product Features

Build only the following initially.

## Users

* Register
* Login
* Get current user

## Products

* Create product
* Get product
* List products
* Product inventory

## Orders

* Create order
* Get order
* List user's orders

## Flash Sale

A product can have limited inventory.

Example:

```text
Product:
Mechanical Keyboard

Inventory:
100 units

Flash sale:
Starts at a specific time
```

Multiple users may attempt to purchase the final few items simultaneously.

This concurrency problem is extremely important.

---

# Database

Create PostgreSQL migrations.

Initial tables:

```text
users
products
orders
order_items
flash_sales
```

Use proper:

* primary keys
* foreign keys
* indexes
* constraints
* timestamps

Do not over-engineer the schema.

I want to understand it.

---

# Architecture Requirements

Keep the Rust application modular.

Use something approximately like:

```text
src/
├── main.rs
├── config/
├── routes/
├── handlers/
├── services/
├── repositories/
├── models/
├── db/
├── errors/
└── middleware/
```

Keep responsibilities clear:

```text
Handler
   ↓
Service
   ↓
Repository
   ↓
PostgreSQL
```

Explain why each layer exists.

Avoid creating abstractions that provide no real benefit.

---

# Extremely Important: Teach Me While Building

Whenever you create an important component, explain:

1. What problem does this solve?
2. Why are we putting it here?
3. What happens internally?
4. What happens when traffic increases?
5. What could become a bottleneck?
6. How will we measure the bottleneck?
7. What would we change later at higher scale?

For example, if creating a database connection pool, explain:

```text
Request
   ↓
Connection Pool
   ↓
PostgreSQL
```

Explain what happens when:

```text
10 requests
100 requests
1,000 requests
10,000 requests
```

Do not just generate code.

I want to understand what the code is doing internally.

---

# Performance Engineering

Create a `docs/performance.md`.

Explain the important metrics we will eventually measure:

* requests per second
* p50 latency
* p95 latency
* p99 latency
* error rate
* CPU usage
* memory usage
* database CPU
* database connections
* query latency
* lock contention

I want to learn how to determine:

> "The system is slow because of X."

rather than simply guessing.

---

# Load Testing

Create a `load-tests/` directory.

Eventually use k6.

Do NOT immediately simulate 10,000 users.

Start with:

```text
10 users
100 users
500 users
1,000 users
```

Then increase gradually.

For every test record:

```text
Concurrent users
Requests/sec
p50
p95
p99
Error rate
CPU
Memory
DB utilization
```

Create a document where we record the results.

---

# Bottleneck Investigation

This is one of the most important parts of the project.

Whenever a load test causes degradation:

DO NOT immediately fix it.

First:

1. Observe the metrics.
2. Identify the suspected bottleneck.
3. Explain why it is the bottleneck.
4. Propose possible solutions.
5. Choose one solution.
6. Implement it.
7. Load-test again.
8. Compare before vs after.

For example:

```text
Before:

RPS: 1500
p95: 800ms
DB CPU: 95%
API CPU: 30%
```

We might conclude:

```text
Database is the bottleneck.
```

Then investigate the actual database issue.

This process should be documented.

---

# Architecture Decision Records

Create:

```text
docs/decisions/
```

Whenever we introduce an important technology, create an ADR.

Examples:

```text
001-postgresql.md
002-redis.md
003-queue.md
004-load-balancer.md
005-kafka.md
006-kubernetes.md
```

Each ADR should answer:

```text
Problem
Options considered
Decision
Why we chose it
Trade-offs
What problem it solves
What new problems it creates
```

This is important because I want to learn architecture decisions rather than memorize technologies.

---

# Rules for You as My Coding Assistant

1. Do not blindly generate large amounts of code.

2. Work incrementally.

3. Before implementing a major component, explain its purpose.

4. Prefer simple solutions first.

5. Do not introduce microservices just because they are popular.

6. Do not introduce Redis until we have a reason.

7. Do not introduce Kafka until we have a reason.

8. Do not introduce Kubernetes until the application actually benefits from it.

9. Do not optimize without measurements.

10. Do not claim the system supports 10k users until we actually load-test it.

11. When something fails under load, help me investigate it rather than immediately hiding the problem with another technology.

12. Keep the application runnable after every stage.

13. Keep documentation updated.

14. Explain important Rust concepts when they relate to backend performance or concurrency.

15. Prefer production-like practices, but keep the implementation understandable for someone learning large-scale systems.

---

# Rust Learning Requirement

Since I am also using this project to become stronger in Rust, explain important Rust concepts when they appear.

Especially:

* ownership
* borrowing
* lifetimes when relevant
* Arc
* Mutex
* async/await
* Tokio
* channels
* Send
* Sync
* concurrency
* task spawning
* connection pools

But don't turn every explanation into a Rust tutorial.

Only explain the concepts relevant to the current backend problem.

---

# First Task

Do NOT build the whole application.

First:

1. Create the project directory.
2. Initialize the Rust project.
3. Create the initial folder structure.
4. Set up Axum.
5. Set up PostgreSQL with Docker Compose.
6. Set up SQLx.
7. Create the initial database migration.
8. Implement a health endpoint.
9. Implement a simple product endpoint.
10. Make sure everything runs locally.
11. Create the initial README.
12. Create `docs/architecture/v1.md`.
13. Create `docs/performance.md`.
14. Create the first ADR explaining why V1 is intentionally a simple monolith.

Then STOP.

Show me:

* the directory structure
* architecture diagram
* files created
* how to run the application
* what each important file does
* what we should test
* what the next step will be

Do not implement future scaling components yet.

The objective is to build this system together step-by-step and eventually discover the scaling architecture through measurement.

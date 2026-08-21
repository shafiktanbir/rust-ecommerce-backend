# ADR-001: V1 is an Intentional Monolith with PostgreSQL

**Date:** 2026-08-21
**Status:** Accepted
**Deciders:** Engineering team

---

## Context

We are building an e-commerce system with a flash-sale feature.

The system must eventually handle:
- 10,000 concurrent users
- High-throughput flash-sale bursts
- Safe inventory management (no overselling)

The question is: **where do we start?**

---

## Options Considered

### Option A: Distributed microservices from day one
- Separate services for: users, products, orders, inventory, notifications
- Kafka for async communication between services
- Kubernetes for orchestration
- Redis for caching

**Why we rejected this:**
1. We have no measurements. We don't know where the bottlenecks are.
2. Distributed systems have failure modes we can't understand without measuring them.
3. The complexity would make it harder to learn what each component actually does.
4. "Build for scale you don't have" → premature optimization.

### Option B: Monolith with PostgreSQL (Chosen)
- Single Rust process
- Single PostgreSQL database
- Docker Compose for local infrastructure
- Clean modular code structure

---

## Decision

**V1 is a modular monolith with PostgreSQL.**

We will run load tests on V1 to identify the actual bottlenecks, then introduce new components
only when measurements justify them.

---

## Why PostgreSQL?

| Property | Value for this system |
|----------|----------------------|
| ACID transactions | Critical for inventory correctness — no overselling |
| Row-level locking | `SELECT FOR UPDATE` for safe concurrent inventory decrements |
| Reliability | Battle-tested, not experimental |
| Developer tooling | pgAdmin, EXPLAIN ANALYZE, pg_stat_statements |
| Constraints | `CHECK (inventory_count >= 0)` as last line of defense |
| JSON support | JSONB columns if we need flexible product attributes later |

---

## Why Rust + Axum?

| Property | Value |
|----------|-------|
| Memory safety | No garbage collector pauses → predictable low latency |
| Zero-cost async | Tokio handles 10k concurrent connections on 8 OS threads |
| Compile-time SQL | sqlx verifies SQL against schema at build time — no runtime surprises |
| Performance | Comparable to C/C++, safer than Go at high concurrency |

---

## Why a Monolith First?

### The Bottleneck Discovery Principle

> You cannot optimize what you cannot measure.

A monolith gives us a clean baseline:
- Simple to instrument
- Easy to reason about failure
- Fast to deploy and iterate
- Forces us to find the *actual* bottleneck before adding complexity

Distributed systems solve scaling problems. But a distributed system that has a misconfigured
connection pool or a missing index will still be slow — just harder to debug.

---

## Trade-offs

| What we gain | What we defer |
|-------------|---------------|
| Simplicity — easy to understand | Horizontal scaling of individual services |
| Fast iteration | Independent deployment of components |
| Clear baseline measurements | Fault isolation between services |
| Easy debugging | Resilience to partial failures |

---

## What Problems This Creates

1. **Single point of failure:** If the Rust process dies, the entire API is down.
   → Fixed in V3 (multiple instances behind a load balancer)

2. **DB contention:** All read and write traffic shares the same connection pool.
   → Measured in V1 load tests, fixed in V5 (read replicas)

3. **No background processing:** Slow operations (email, analytics) block request handlers.
   → Fixed in V4 (background job queue)

4. **No caching:** Every request hits PostgreSQL.
   → Fixed in V2 (Redis) after we measure cache-miss rate

---

## What We Will Measure in V1

Before moving to V2:

```
□ Maximum RPS on GET /products
□ p95 latency at 100, 500, 1000 concurrent users
□ CPU utilization of the Rust process at peak
□ PostgreSQL connection count at peak
□ What fails first when we push beyond capacity
□ Inventory correctness under 100 concurrent purchase attempts
```

---

## Next Decision Point

When V1 load tests show one of these conditions:
- p95 > 200ms at moderate load
- DB CPU > 70% sustained
- Pool exhaustion errors in logs
- Inventory overselling under concurrent load

→ We will write ADR-002 for the next architectural change.

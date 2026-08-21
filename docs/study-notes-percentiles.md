# Performance Metrics — Study Notes

> Companion notes for the Rust E-Commerce Scaling Lab.
> Topic: Understanding latency percentiles and how to read them.

---

## 1. What is Latency?

**Latency** = how long a request takes from the moment the user sends it
to the moment they get a response back.

```
User sends request → [server processes it] → User gets response
|_______________ latency __________________|

Low latency  = fast response   = good
High latency = slow response   = bad
```

---

## 2. Why Average is Useless

Imagine 10 requests with these response times:

```
5ms, 6ms, 4ms, 5ms, 6ms, 5ms, 4ms, 5ms, 6ms, 1000ms
```

**Average = (5+6+4+5+6+5+4+5+6+1000) / 10 = 105ms**

The average says "105ms" — but **9 out of 10 users** got under 10ms.
One slow request ruined the average.

The average **hides the real experience**.

---

## 3. Percentiles — The Right Tool

Sort all responses from fastest to slowest:

```
Position:   1    2    3    4    5    6    7    8    9    10
Time(ms):   4    4    5    5    5    5    6    6    6   1000
                                    ↑                   ↑
                                   p50                 p99
```

### p50 (50th percentile = median)
> "The request at position 50 out of 100"
> = "Half of users are faster than this, half are slower"
> = The typical, everyday experience

### p95 (95th percentile)
> "The request at position 95 out of 100"
> = "95% of users are faster than this"
> = What almost everyone experiences
> = **The most important metric for engineers**

### p99 (99th percentile)
> "The request at position 99 out of 100"
> = "Only 1% of users are slower than this"
> = Catches hidden problems
> = At 10,000 users/hour → 100 people have this bad experience every hour

### p99.9 (99.9th percentile)
> = 1 in 1,000 requests
> = Only matters at very high scale (millions of requests/day)

---

## 4. Visual Summary

```
100 requests sorted by response time:

[====fast====][=====normal=====][===slow===][!!very slow!!]
 1         50                95          99              100
              ↑               ↑           ↑
             p50             p95         p99

p50 = "typical user"
p95 = "almost everyone" ← engineers watch this most
p99 = "worst users"     ← spikes here = something is wrong
```

---

## 5. What "Spiking" Means

**Healthy system** (low traffic):
```
p50:  5ms
p95:  8ms   ← close to p50, small gap = consistent
p99: 12ms
```

**Unhealthy system** (pool exhausted, high traffic):
```
p50:  250ms
p95: 1200ms  ← huge jump from p50 = requests are WAITING
p99: 28000ms ← some timing out completely
```

The **gap between p50 and p95** is your early warning signal.
Small gap = healthy. Large gap = requests are queuing somewhere.

---

## 6. The Connection Pool Link

Our pool has **10 connections**.

```
Scenario: 1000 users hit the server at the same time

Step 1: Tokio accepts all 1000 connections (handles this fine)
Step 2: All 1000 try to borrow from pool
Step 3: 10 get connections immediately → 5ms response
Step 4: 990 wait in queue
Step 5: Every 10ms, 10 connections free up → 10 more served
Step 6: User #990 waited 99 × 10ms = 990ms before even starting query

Result:
  p50:  500ms  (middle user waited ~50 rounds × 10ms)
  p95:  950ms  (95th user waited ~95 rounds × 10ms)
  p99: ~990ms  (near timeout)
```

**p95 spiking = requests are sitting in the pool queue.**

---

## 7. How to Fix It (the V2 decision)

When k6 shows p95 spiking:

**Option A — Increase pool size:**
```rust
.max_connections(50)  // was 10
```
Risk: PostgreSQL default max_connections = 100.
3 server instances × 50 = 150 → PostgreSQL refuses connections.

**Option B — Optimize queries (reduce time per connection):**
```sql
-- Add missing index so query goes from 10ms → 1ms
CREATE INDEX idx_products_name ON products(name);
```
Effect: each connection now serves 10× more requests.

**Option C — Add a read replica (V5):**
```
All SELECT (reads)  → replica database (copy)
All INSERT/UPDATE   → primary database
```
Doubles your read capacity without touching pool size.

**The rule: measure first, then pick the right option.**

---

## 8. Targets to Aim For

| Environment | p50 | p95 | p99 |
|------------|-----|-----|-----|
| Simple CRUD API | < 10ms | < 50ms | < 200ms |
| Complex queries | < 50ms | < 200ms | < 500ms |
| Flash sale endpoint | < 5ms | < 20ms | < 100ms |
| "Something is wrong" threshold | > 100ms | > 500ms | > 2000ms |

---

## 9. Reading k6 Output

When we run:
```bash
k6 run --vus 100 --duration 60s load-tests/v1_products.js
```

k6 outputs:
```
http_req_duration:
    avg=12.4ms   min=2ms    med=8ms
    max=1200ms   p(90)=18ms p(95)=45ms  p(99)=200ms
```

Read it as:
- `avg=12ms` → ignore, misleading
- `med=8ms` → same as p50, most requests take 8ms
- `p(95)=45ms` → 95% of users get under 45ms ← watch this number
- `p(99)=200ms` → worst 1% wait 200ms
- `max=1200ms` → someone had a bad day, probably outlier

**The number that changes most under load = your bottleneck.**

---

## 10. Quick Reference

```
percentile = "what % of requests were faster than this?"

p50  = 50% faster  = typical user
p95  = 95% faster  = almost everyone
p99  = 99% faster  = worst 1%

spike = sudden jump under higher load
gap   = difference between p50 and p95 (small = healthy)
```

---

## 11. Best Resources to Study Further

### Free Blogs

| Resource | What it covers | Link |
|----------|---------------|------|
| **High Scalability** | Real case studies (how Discord, Slack scaled) | highscalability.com |
| **Brendan Gregg's Blog** | Deep performance engineering, latency explained | brendangregg.com |
| **Martin Fowler** | Architecture patterns, database bottlenecks | martinfowler.com |
| **The USE Method** | CPU/memory/IO bottleneck framework | brendangregg.com/usemethod.html |

### Specific Articles Worth Reading

1. **"How NOT to Measure Latency"** by Gil Tene
   - Search: `"how not to measure latency" Gil Tene`
   - Explains exactly why averages lie and why percentiles matter

2. **"Everything you know about latency is wrong"** by Brambling
   - Search: `"everything you know about latency is wrong" mechanical sympathy`

3. **k6 documentation — Results interpretation**
   - `k6.io/docs/using-k6/metrics/`

### Books (if you go deep)
- **"Designing Data-Intensive Applications"** by Martin Kleppmann
  - Chapter 1 covers latency, percentiles, SLOs
  - The single best book for what we're building in this lab

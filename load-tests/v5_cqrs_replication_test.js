import { check, sleep } from 'k6';
import http from 'k6/http';
import { Trend, Counter } from 'k6/metrics';

// ─────────────────────────────────────────────────────────────────────────────
// Milestone V5 Load Test — PostgreSQL Read Replicas, CQRS & Read-Your-Own-Writes
// Target: http://localhost:8080 (Nginx Load Balancer → 3x Axum API Workers)
// Goals:
//   1. Stress test CQRS Read-Write Splitting (Reads -> Replica DB, Writes -> Primary DB).
//   2. Verify Read-Your-Own-Writes Consistency: User creates order, immediately fetches
//      GET /orders within 50ms, and asserts that the new order is visible.
//   3. Monitor replication lag and database pool health via GET /health/db.
// ─────────────────────────────────────────────────────────────────────────────

const readDuration = new Trend('read_duration');
const writeDuration = new Trend('write_duration');
const readAfterWriteDuration = new Trend('read_after_write_duration');
const ryowSuccessCounter = new Counter('ryow_success_count');
const ryowFailCounter = new Counter('ryow_fail_count');

export const options = {
  stages: [
    { duration: '10s', target: 100 },  // Warmup to 100 VUs
    { duration: '20s', target: 500 },  // Scale to 500 VUs
    { duration: '20s', target: 1000 }, // Peak Stress: 1,000 VUs
    { duration: '10s', target: 0 },    // Cool down
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],             // 99%+ Success rate requirement
    read_duration: ['p(95)<50'],                // Sub-50ms p95 read latency
    read_after_write_duration: ['p(95)<100'],   // Sub-100ms p95 for immediate read-after-write
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

export function setup() {
  const headers = { 'Content-Type': 'application/json' };
  const timestamp = Date.now();

  // 1. Seed a sample product for testing catalog reads
  const productPayload = JSON.stringify({
    name: `V5 Replication Mouse - ${timestamp}`,
    price: 89.99,
    inventory_count: 1000000,
  });

  const prodRes = http.post(`${BASE_URL}/products`, productPayload, { headers });
  let productId = null;
  if (prodRes.status === 201) {
    const body = JSON.parse(prodRes.body);
    productId = body.id;
  }

  return { productId };
}

export default function (data) {
  const headers = { 'Content-Type': 'application/json' };
  const vuId = __VU;
  const iterId = __ITER;

  // ── Step 1: User Registration & Login ─────────────────────────────────────
  const email = `v5_user_${vuId}_${iterId}_${Date.now()}@example.com`;
  const registerPayload = JSON.stringify({
    email: email,
    password: 'Password123!',
  });

  const regRes = http.post(`${BASE_URL}/auth/register`, registerPayload, { headers });
  check(regRes, {
    'register status is 201': (r) => r.status === 201,
  });

  if (regRes.status !== 201) return;

  const authData = JSON.parse(regRes.body);
  const token = authData.token;
  const authHeaders = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`,
  };

  // ── Step 2: Catalog Reads (Routed to Read Replica DB) ──────────────────────
  const startTimeRead = Date.now();
  const catalogRes = http.get(`${BASE_URL}/products?limit=10`, { headers });
  readDuration.add(Date.now() - startTimeRead);

  check(catalogRes, {
    'catalog read status is 200': (r) => r.status === 200,
  });

  // ── Step 3: Create Order (Routed to Primary DB + Sets 5s Sticky Flag) ──────
  if (data.productId) {
    const orderPayload = JSON.stringify({
      items: [
        {
          product_id: data.productId,
          quantity: 1,
        },
      ],
    });

    const startWriteTime = Date.now();
    const orderRes = http.post(`${BASE_URL}/orders`, orderPayload, { headers: authHeaders });
    writeDuration.add(Date.now() - startWriteTime);

    const orderSuccess = check(orderRes, {
      'order status is 201': (r) => r.status === 201,
    });

    if (orderSuccess) {
      const createdOrder = JSON.parse(orderRes.body);
      const createdOrderId = createdOrder.id;

      // ── Step 4: Immediate Read-After-Write Verification ─────────────────
      // Fetch GET /orders within milliseconds to verify Sticky Session routing to Primary DB!
      const startRyowTime = Date.now();
      const userOrdersRes = http.get(`${BASE_URL}/orders`, { headers: authHeaders });
      readAfterWriteDuration.add(Date.now() - startRyowTime);

      const ryowCheck = check(userOrdersRes, {
        'RYOW status is 200': (r) => r.status === 200,
        'RYOW returns created order immediately': (r) => {
          if (r.status !== 200) return false;
          const userOrders = JSON.parse(r.body);
          return Array.isArray(userOrders) && userOrders.some((o) => o.id === createdOrderId);
        },
      });

      if (ryowCheck) {
        ryowSuccessCounter.add(1);
      } else {
        ryowFailCounter.add(1);
      }
    }
  }

  // ── Step 5: Check Database Cluster Health & Lag Metrics ────────────────────
  if (iterId % 20 === 0) {
    const healthRes = http.get(`${BASE_URL}/health/db`, { headers });
    check(healthRes, {
      'health db status is 200': (r) => r.status === 200,
    });
  }

  sleep(0.1);
}

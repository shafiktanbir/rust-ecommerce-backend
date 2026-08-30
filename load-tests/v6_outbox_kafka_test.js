import { check, sleep } from 'k6';
import http from 'k6/http';
import { Trend, Counter } from 'k6/metrics';

// ─────────────────────────────────────────────────────────────────────────────
// Milestone V6 Realistic Load Test — Atomic Outbox & Kafka Streaming
// Target: http://localhost:8080 (Nginx Load Balancer → 3x Axum API Workers)
// SLA Requirements:
//   1. Pre-authenticates user pool in setup() to avoid bcrypt CPU thrashing.
//   2. Measures atomic PostgreSQL Order + Outbox insertion under 1,500 VUs.
//   3. Validates 99%+ HTTP success rate and sub-50ms p95 latency.
// ─────────────────────────────────────────────────────────────────────────────

const checkoutDuration = new Trend('checkout_duration');
const catalogReadDuration = new Trend('catalog_read_duration');
const successfulOrders = new Counter('successful_orders_count');
const failedOrders = new Counter('failed_orders_count');

export const options = {
  stages: [
    { duration: '10s', target: 100 },  // Warmup to 100 VUs
    { duration: '20s', target: 500 },  // Scale to 500 VUs
    { duration: '20s', target: 1500 }, // Peak Stress: 1,500 VUs
    { duration: '10s', target: 0 },    // Cool down
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],            // 99%+ Success rate SLA
    checkout_duration: ['p(95)<100'],          // Sub-100ms p95 for atomic outbox checkout
    catalog_read_duration: ['p(95)<30'],       // Sub-30ms p95 for catalog reads
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

export function setup() {
  const headers = { 'Content-Type': 'application/json' };
  const timestamp = Date.now();

  // 1. Seed test product
  const productPayload = JSON.stringify({
    name: `V6 Kafka Keyboard - ${timestamp}`,
    price: 149.99,
    inventory_count: 10000000,
  });

  const prodRes = http.post(`${BASE_URL}/products`, productPayload, { headers });
  let productId = null;
  if (prodRes.status === 201) {
    const body = JSON.parse(prodRes.body);
    productId = body.id;
  }

  // 2. Pre-seed pool of 20 authenticated user tokens to prevent bcrypt CPU thrashing during load test
  const tokens = [];
  for (let i = 0; i < 20; i++) {
    const email = `v6_bench_user_${timestamp}_${i}@example.com`;
    const regPayload = JSON.stringify({ email: email, password: 'Password123!' });
    const regRes = http.post(`${BASE_URL}/auth/register`, regPayload, { headers });
    if (regRes.status === 201) {
      const authData = JSON.parse(regRes.body);
      tokens.push(authData.token);
    }
  }

  return { productId, tokens };
}

export default function (data) {
  const headers = { 'Content-Type': 'application/json' };
  const vuId = __VU;

  // Select token from pre-seeded pool
  const token = data.tokens && data.tokens.length > 0 ? data.tokens[vuId % data.tokens.length] : null;
  const authHeaders = token ? {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${token}`,
  } : headers;

  // ── Step 1: Catalog Read (Routed to Replica DB) ───────────────────────────
  const startRead = Date.now();
  const catalogRes = http.get(`${BASE_URL}/products?limit=10`, { headers });
  catalogReadDuration.add(Date.now() - startRead);

  check(catalogRes, {
    'catalog read status 200': (r) => r.status === 200,
  });

  // ── Step 2: Atomic Order Checkout (POST /orders) ───────────────────────────
  if (data.productId && token) {
    const orderPayload = JSON.stringify({
      items: [
        {
          product_id: data.productId,
          quantity: 1,
        },
      ],
    });

    const startCheckout = Date.now();
    const orderRes = http.post(`${BASE_URL}/orders`, orderPayload, { headers: authHeaders });
    checkoutDuration.add(Date.now() - startCheckout);

    const isSuccess = check(orderRes, {
      'order status 201': (r) => r.status === 201,
    });

    if (isSuccess) {
      successfulOrders.add(1);
    } else {
      failedOrders.add(1);
    }
  }

  sleep(0.05);
}

import { check, sleep } from 'k6';
import http from 'k6/http';
import { Trend, Counter } from 'k6/metrics';

// ─────────────────────────────────────────────────────────────────────────────
// Order Creation Choke Test — 500 VU Concurrent Order Checkout
// Target: http://localhost:8080 (Nginx Load Balancer → 3x Axum API Workers)
// Goal: Measure HTTP latency, connection pool timeouts, and failure rate
//       when order creation handles 250ms synchronous in-band side effects.
// ─────────────────────────────────────────────────────────────────────────────

const orderDuration = new Trend('order_duration');
const failedOrders = new Counter('failed_orders');

export const options = {
  stages: [
    { duration: '10s', target: 100 },   // Warmup to 100 VUs
    { duration: '15s', target: 300 },   // Ramp up to 300 VUs
    { duration: '25s', target: 500 },   // Max Stress: Hold 500 VUs
    { duration: '10s', target: 0 },     // Ramp down
  ],
  thresholds: {
    http_req_failed: ['rate<0.05'],     // We expect failures under choke conditions
    order_duration: ['p(95)<1000'],    // Track p95 SLA degradation
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

export function setup() {
  const headers = { 'Content-Type': 'application/json' };

  // 1. Register test user to acquire JWT Token
  const timestamp = Date.now();
  const registerPayload = JSON.stringify({
    email: `choke_user_${timestamp}@example.com`,
    password: 'Password123!',
  });
  
  let token = null;
  const regRes = http.post(`${BASE_URL}/auth/register`, registerPayload, { headers });

  if (regRes.status === 201 || regRes.status === 200) {
    const body = JSON.parse(regRes.body);
    token = body.token;
  } else {
    const loginRes = http.post(`${BASE_URL}/auth/login`, registerPayload, { headers });
    if (loginRes.status === 200) {
      const body = JSON.parse(loginRes.body);
      token = body.token;
    }
  }

  // 2. Seed 5 products for order testing
  const productIds = [];
  for (let i = 1; i <= 5; i++) {
    const productPayload = JSON.stringify({
      name: `Choke Test Item ${i} - ${timestamp}`,
      price: 29.99 * i,
      inventory_count: 500000,
    });
    const pRes = http.post(`${BASE_URL}/products`, productPayload, { headers });
    if (pRes.status === 201 || pRes.status === 200) {
      const body = JSON.parse(pRes.body);
      productIds.push(body.id);
    }
  }

  return {
    token: token,
    productIds: productIds,
  };
}

export default function (data) {
  if (!data.token || !data.productIds || data.productIds.length === 0) {
    return;
  }

  const authHeaders = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  const selectedProductId = data.productIds[Math.floor(Math.random() * data.productIds.length)];
  const orderPayload = JSON.stringify({
    items: [
      {
        product_id: selectedProductId,
        quantity: 1,
      },
    ],
  });

  const start = Date.now();
  const res = http.post(`${BASE_URL}/orders`, orderPayload, { headers: authHeaders });
  const elapsed = Date.now() - start;
  orderDuration.add(elapsed);

  const isSuccess = check(res, {
    'order status is 201': (r) => r.status === 201,
  });

  if (!isSuccess) {
    failedOrders.add(1);
  }

  // Minimal think time between checkout attempts (50ms - 150ms)
  sleep(0.05 + Math.random() * 0.1);
}

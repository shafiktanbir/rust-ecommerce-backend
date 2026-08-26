import { check, sleep } from 'k6';
import http from 'k6/http';
import { Trend } from 'k6/metrics';

// ─────────────────────────────────────────────────────────────────────────────
// Milestone V4 Realistic Load Test — Mixed E-Commerce Production Workload
// Target: http://localhost:8080 (Nginx Load Balancer → 3x Axum API Workers)
// Workload Mix:
//   - 70% Catalog Reads (Cached GET /products)
//   - 15% Product Detail Lookups (Random ID GET /products/:id)
//   - 15% Order Creation Checkouts (Authenticated POST /orders with JWT + DB Transaction)
// ─────────────────────────────────────────────────────────────────────────────

const readDuration = new Trend('read_duration');
const writeDuration = new Trend('write_duration');

export const options = {
  stages: [
    { duration: '15s', target: 200 },   // Warmup to 200 VUs
    { duration: '30s', target: 1000 },  // Ramp up to 1,000 VUs
    { duration: '45s', target: 2000 },  // Peak Stress: Hold 2,000 VUs
    { duration: '15s', target: 0 },     // Cool down
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],    // SLA: <1% failure rate
    http_req_duration: ['p(95)<500'],  // SLA: 95% of total requests < 500ms
    read_duration: ['p(95)<200'],      // SLA: 95% of reads < 200ms
    write_duration: ['p(95)<600'],     // SLA: 95% of order writes < 600ms
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

export function setup() {
  const headers = { 'Content-Type': 'application/json' };

  // 1. Register test user to acquire JWT Token
  const timestamp = Date.now();
  const registerPayload = JSON.stringify({
    email: `k6_benchmark_user_${timestamp}@example.com`,
    password: 'Password123!',
  });
  
  let token = null;
  const regRes = http.post(`${BASE_URL}/auth/register`, registerPayload, { headers });

  if (regRes.status === 201 || regRes.status === 200) {
    const body = JSON.parse(regRes.body);
    token = body.token;
  } else {
    // Fallback: Login if user already exists
    const loginRes = http.post(`${BASE_URL}/auth/login`, registerPayload, { headers });
    if (loginRes.status === 200) {
      const body = JSON.parse(loginRes.body);
      token = body.token;
    }
  }

  // 2. Seed 5 products for realistic catalogue testing
  const productIds = [];
  for (let i = 1; i <= 5; i++) {
    const productPayload = JSON.stringify({
      name: `V4 Benchmark Item ${i} - ${timestamp}`,
      price: 49.99 * i,
      inventory_count: 100000,
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
  const rand = Math.random();
  const headers = { 'Content-Type': 'application/json' };

  if (rand < 0.70) {
    // -------------------------------------------------------------------------
    // WORKLOAD 1: 70% Catalog Reads (Cached GET /products)
    // -------------------------------------------------------------------------
    const start = Date.now();
    const offset = Math.floor(Math.random() * 3) * 20;
    const res = http.get(`${BASE_URL}/products?limit=20&offset=${offset}`);
    readDuration.add(Date.now() - start);

    check(res, {
      'catalog read status is 200': (r) => r.status === 200,
    });

  } else if (rand < 0.85) {
    // -------------------------------------------------------------------------
    // WORKLOAD 2: 15% Product Detail Lookups (GET /products/:id)
    // -------------------------------------------------------------------------
    if (data.productIds && data.productIds.length > 0) {
      const targetId = data.productIds[Math.floor(Math.random() * data.productIds.length)];
      const start = Date.now();
      const res = http.get(`${BASE_URL}/products/${targetId}`);
      readDuration.add(Date.now() - start);

      check(res, {
        'single product status is 200': (r) => r.status === 200,
      });
    }

  } else {
    // -------------------------------------------------------------------------
    // WORKLOAD 3: 15% Order Creation (JWT Auth + DB Write Transaction)
    // -------------------------------------------------------------------------
    if (data.token && data.productIds && data.productIds.length > 0) {
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
      writeDuration.add(Date.now() - start);

      check(res, {
        'order status is 201': (r) => r.status === 201,
      });
    }
  }

  // Realistic User Think Time: 1 to 2.5 seconds pause between actions
  sleep(1.0 + Math.random() * 1.5);
}

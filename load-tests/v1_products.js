import { check, sleep } from 'k6';
import http from 'k6/http';

// ─────────────────────────────────────────────────────────────────────────────
// Load Test Configuration — V1 Baseline Test
//
// Stage 1: Ramp up to 10 VUs over 10s (warm-up)
// Stage 2: Ramp up to 100 VUs over 20s (moderate load)
// Stage 3: Hold 100 VUs for 30s (steady state measurement)
// Stage 4: Ramp up to 300 VUs over 20s (stress test)
// Stage 5: Ramp down to 0 VUs over 10s (cooldown)
// ─────────────────────────────────────────────────────────────────────────────
export const options = {
  stages: [
    { duration: '10s', target: 10 },
    { duration: '20s', target: 100 },
    { duration: '30s', target: 100 },
    { duration: '20s', target: 300 },
    { duration: '10s', target: 0 },
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'], // <1% errors allowed
    http_req_duration: ['p(95)<200'], // 95% of requests must finish within 200ms
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

export function setup() {
  // Seed a product before running tests
  const payload = JSON.stringify({
    name: 'Stress Test Mechanical Keyboard',
    price: 149.99,
    inventory_count: 5000,
  });
  const headers = { 'Content-Type': 'application/json' };
  const res = http.post(`${BASE_URL}/products`, payload, { headers });

  if (res.status === 201 || res.status === 200) {
    const body = JSON.parse(res.body);
    return { productId: body.id };
  }
  return { productId: null };
}

export default function (data) {
  // 1. Health check
  const healthRes = http.get(`${BASE_URL}/health`);
  check(healthRes, {
    'health check status is 200': (r) => r.status === 200,
  });

  // 2. Fetch product catalog (Read heavy — standard ecommerce load)
  const listRes = http.get(`${BASE_URL}/products?limit=20&offset=0`);
  check(listRes, {
    'list products status is 200': (r) => r.status === 200,
    'list products response is array': (r) => Array.isArray(JSON.parse(r.body)),
  });

  // 3. Fetch single product if seeded
  if (data.productId) {
    const singleRes = http.get(`${BASE_URL}/products/${data.productId}`);
    check(singleRes, {
      'get single product status is 200': (r) => r.status === 200,
    });
  }

  // Think time between user interactions (100ms - 300ms)
  sleep(0.1 + Math.random() * 0.2);
}

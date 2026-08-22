import { check, sleep } from 'k6';
import http from 'k6/http';

// ─────────────────────────────────────────────────────────────────────────────
// Milestone V3 Load Test — 3,000 VU Cluster Stress Test
// Target: http://localhost:8080 (Nginx Load Balancer → 3x Axum API Workers)
// ─────────────────────────────────────────────────────────────────────────────
export const options = {
  stages: [
    { duration: '10s', target: 200 },   // Warmup to 200 VUs
    { duration: '15s', target: 1500 },  // Ramp up to 1,500 VUs
    { duration: '30s', target: 3000 },  // Max Stress: Hold 3,000 VUs
    { duration: '10s', target: 0 },     // Ramp down
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],    // SLA: <1% failure rate
    http_req_duration: ['p(95)<200'],  // SLA: 95% of requests < 200ms
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

export function setup() {
  const payload = JSON.stringify({
    name: 'V3 Cluster Benchmark Keyboard',
    price: 199.99,
    inventory_count: 50000,
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
  // 1. Product list catalog read
  const listRes = http.get(`${BASE_URL}/products?limit=20&offset=0`);
  check(listRes, {
    'list status is 200': (r) => r.status === 200,
  });

  // 2. Single product detail read
  if (data.productId) {
    const singleRes = http.get(`${BASE_URL}/products/${data.productId}`);
    check(singleRes, {
      'single product status is 200': (r) => r.status === 200,
    });
  }

  // Think time between requests (50ms - 150ms)
  sleep(0.05 + Math.random() * 0.1);
}

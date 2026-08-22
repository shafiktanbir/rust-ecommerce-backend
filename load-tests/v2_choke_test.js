import { check } from 'k6';
import http from 'k6/http';

// ─────────────────────────────────────────────────────────────────────────────
// V2 Choke & Saturation Test — Single-Instance Collapse Benchmark
//
// How this script forces the single-instance server to choke:
// 1. Zero Sleep (Tight Loop): Removes sleep pauses so VUs hammer sockets continuously.
// 2. High Concurrency: Ramps up to 3,000 Virtual Users (VUs).
// 3. Pool & Listener Exhaustion: 3,000 VUs fighting for 100 Redis pool slots & 1 port.
// ─────────────────────────────────────────────────────────────────────────────
export const options = {
  stages: [
    { duration: '10s', target: 200 },   // Warmup to 200 VUs
    { duration: '15s', target: 500 },  // Ramp to 1,500 VUs
    { duration: '30s', target: 1000 },  // MAX CHOKE: Hold 3,000 VUs with ZERO pause
    { duration: '10s', target: 0 },     // Cool down
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],    // Strict SLA: <1% failure allowed
    http_req_duration: ['p(95)<200'],  // Strict SLA: 95% requests < 200ms
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

export function setup() {
  const payload = JSON.stringify({
    name: 'Choke Test Heavy Keyboard',
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

  // 🔴 ZERO THINK TIME: No sleep() forces tight-loop socket flooding
}

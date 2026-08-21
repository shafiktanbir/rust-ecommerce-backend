import { check, sleep } from 'k6';
import http from 'k6/http';

export const options = {
  stages: [
    { duration: '10s', target: 50 },
    { duration: '20s', target: 500 },
    { duration: '30s', target: 500 },
    { duration: '10s', target: 0 },
  ],
  thresholds: {
    http_req_failed: ['rate<0.01'],
    http_req_duration: ['p(95)<200'],
  },
};

const BASE_URL = __ENV.TARGET_URL || 'http://localhost:8080';

export function setup() {
  const payload = JSON.stringify({
    name: 'V2 Redis Cache Test Product',
    price: 99.99,
    inventory_count: 10000,
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
  const listRes = http.get(`${BASE_URL}/products?limit=20&offset=0`);
  check(listRes, {
    'list products status is 200': (r) => r.status === 200,
  });

  if (data.productId) {
    const singleRes = http.get(`${BASE_URL}/products/${data.productId}`);
    check(singleRes, {
      'get single product status is 200': (r) => r.status === 200,
    });
  }

  sleep(0.05 + Math.random() * 0.1);
}

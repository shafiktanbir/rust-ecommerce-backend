import { test, expect } from '@playwright/test';

test.describe('Health Check API E2E Tests', () => {
  test('GET /health - should return 200 OK with health status metadata', async ({ request }) => {
    const response = await request.get('/health');
    
    expect(response.status()).toBe(200);
    expect(response.headers()['content-type']).toContain('application/json');

    const body = await response.json();
    expect(body).toMatchObject({
      status: 'ok',
      version: 'v1',
      service: 'ecommerce-lab',
    });
  });
});

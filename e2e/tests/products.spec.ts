import { test, expect } from '@playwright/test';

test.describe.serial('Products API End-to-End Lifecycle', () => {
  let createdProductId: string;

  const testProduct = {
    name: `E2E Test Mechanical Keyboard ${Date.now()}`,
    price: 129.99,
    inventory_count: 50,
  };

  test('POST /products - should create a new product', async ({ request }) => {
    const response = await request.post('/products', {
      data: testProduct,
    });

    expect(response.status()).toBe(201);
    expect(response.headers()['content-type']).toContain('application/json');

    const body = await response.json();
    expect(body).toHaveProperty('id');
    expect(typeof body.id).toBe('string');
    expect(body.name).toBe(testProduct.name);
    expect(body.price).toBe(testProduct.price);
    expect(body.inventory_count).toBe(testProduct.inventory_count);

    // Save generated UUID for subsequent tests
    createdProductId = body.id;
  });

  test('GET /products/:id - should retrieve product by ID', async ({ request }) => {
    expect(createdProductId).toBeDefined();

    const response = await request.get(`/products/${createdProductId}`);

    expect(response.status()).toBe(200);
    const body = await response.json();

    expect(body.id).toBe(createdProductId);
    expect(body.name).toBe(testProduct.name);
    expect(body.price).toBe(testProduct.price);
  });

  test('GET /products - should list products including created item', async ({ request }) => {
    const response = await request.get('/products?limit=50&offset=0');

    expect(response.status()).toBe(200);
    const products = await response.json();

    expect(Array.isArray(products)).toBe(true);
    const matched = products.find((p: { id: string }) => p.id === createdProductId);
    expect(matched).toBeDefined();
    expect(matched.name).toBe(testProduct.name);
  });

  test('GET /products/:id - should return 404 for non-existent product ID', async ({ request }) => {
    const randomUuid = '00000000-0000-0000-0000-000000000000';
    const response = await request.get(`/products/${randomUuid}`);

    expect(response.status()).toBe(404);
  });

  test('POST /products - should reject invalid request body with 422 or 400', async ({ request }) => {
    const invalidPayload = {
      name: 'Missing Price & Inventory',
    };

    const response = await request.post('/products', {
      data: invalidPayload,
    });

    // Axum Json extractor returns 422 Unprocessable Entity for missing required fields
    expect([400, 422]).toContain(response.status());
  });
});

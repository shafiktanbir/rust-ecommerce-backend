# Ecommerce Lab — End-to-End (E2E) Test Suite

This directory contains Playwright API E2E tests for the Rust Axum Ecommerce application.

## Prerequisites

- Node.js (v18+)
- Running Rust Backend (`RUST_LOG=info cargo run --release`) on `http://localhost:8080` (or PostgreSQL running via Docker Compose)

## Installation

```bash
cd e2e
npm install
```

## Running Tests

Run all end-to-end API tests:
```bash
npx playwright test
```

Run tests against a custom API URL:
```bash
TARGET_URL=http://localhost:8080 npx playwright test
```

Run tests with interactive UI mode:
```bash
npx playwright test --ui
```

View HTML Test Report:
```bash
npx playwright show-report
```

## Test Structure

- `playwright.config.ts`: Base URL and reporter configuration.
- `tests/health.spec.ts`: Tests `GET /health` endpoint status and metadata.
- `tests/products.spec.ts`: End-to-end lifecycle testing for products (`POST`, `GET /products/:id`, `GET /products`, 404/422 handling).

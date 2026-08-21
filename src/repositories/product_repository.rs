// src/repositories/product_repository.rs
//
// WHY THIS EXISTS:
//   The repository layer owns ALL direct database access.
//   No other layer should ever write a SQL query.
//
//   This separation means:
//     - SQL is in one place → easy to audit, optimize, and test
//     - If we later switch from PostgreSQL to CockroachDB or add a read replica,
//       only this file changes — handlers and services are unaffected
//     - We can mock the repository in unit tests without needing a real database
//
// WHAT HAPPENS AT SCALE:
//   At 10k users, we will add:
//     - Read replicas: SELECT queries go to replica, writes stay on primary
//     - Query result caching (Redis): frequently read products cached in memory
//     - Database indexes: EXPLAIN ANALYZE will show us which queries are doing seq scans
//
// RUST CONCEPT — async fn:
//   Every function here is `async`. Rust's async model uses Tokio's thread pool.
//   When a query is executing (waiting for PostgreSQL I/O), the Tokio runtime parks
//   that task and runs other tasks on the same OS thread. This is why one Rust process
//   can handle thousands of concurrent requests on just a few OS threads.
//
// RUST CONCEPT — sqlx::query_as!:
//   This macro verifies the SQL at COMPILE TIME against the actual database schema.
//   If a column name changes, your code fails to compile — not at runtime.
//   This requires DATABASE_URL to be set during `cargo build`.
//
// NOTE ON price TYPE:
//   The products.price column is FLOAT8 (double precision) in V1.
//   FLOAT8 maps natively to Rust's f64 — no extra sqlx features needed.
//   V2+ will switch to NUMERIC + rust_decimal for exact monetary arithmetic.

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::product::{CreateProductRequest, Product},
};

/// Fetch a single product by its UUID primary key.
pub async fn get_product(pool: &PgPool, id: Uuid) -> AppResult<Option<Product>> {
    let product = sqlx::query_as!(
        Product,
        r#"
        SELECT id, name, description, price, inventory_count, created_at, updated_at
        FROM products
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(product)
}

/// List all products with a simple limit/offset pagination.
///
/// V1: No cursor-based pagination yet. At scale (millions of products),
/// OFFSET becomes slow because PostgreSQL must skip rows. We'll fix this in V4+.
pub async fn list_products(pool: &PgPool, limit: i64, offset: i64) -> AppResult<Vec<Product>> {
    let products = sqlx::query_as!(
        Product,
        r#"
        SELECT id, name, description, price, inventory_count, created_at, updated_at
        FROM products
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(products)
}

/// Insert a new product and return the created record.
///
/// We use INSERT ... RETURNING to get the server-generated id, timestamps etc.
/// in a single round-trip instead of INSERT then SELECT.
pub async fn create_product(
    pool: &PgPool,
    req: &CreateProductRequest,
) -> AppResult<Product> {
    let product = sqlx::query_as!(
        Product,
        r#"
        INSERT INTO products (id, name, description, price, inventory_count, created_at, updated_at)
        VALUES (gen_random_uuid(), $1, $2, $3, $4, NOW(), NOW())
        RETURNING id, name, description, price, inventory_count, created_at, updated_at
        "#,
        req.name,
        req.description,
        req.price,
        req.inventory_count
    )
    .fetch_one(pool)
    .await?;

    Ok(product)
}

/// Atomically decrement inventory by 1 for a product.
///
/// WHY THIS UPDATE PATTERN MATTERS (Flash Sale Critical Section):
///   When 100 users simultaneously try to buy the last item:
///
///   WITHOUT atomic check:
///     - User 1 reads inventory = 1, decides to buy
///     - User 2 reads inventory = 1, decides to buy
///     - Both decrement → inventory = -1 (oversold!)
///
///   WITH this atomic UPDATE ... WHERE inventory_count > 0:
///     - PostgreSQL uses row-level locking internally
///     - Only ONE update succeeds per decrement
///     - Returns the updated row if successful, nothing if out of stock
///     - No race condition possible at the DB level
///
///   This is correct but creates lock contention at high concurrency.
///   We will measure this with k6 in V2.
pub async fn decrement_inventory(pool: &PgPool, product_id: Uuid) -> AppResult<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE products
        SET inventory_count = inventory_count - 1,
            updated_at = NOW()
        WHERE id = $1
          AND inventory_count > 0
        RETURNING id
        "#,
        product_id
    )
    .fetch_optional(pool)
    .await?;

    // Returns true if a row was updated (inventory was available), false if out of stock
    Ok(result.is_some())
}

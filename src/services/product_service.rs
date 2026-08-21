// src/services/product_service.rs
//
// WHY THIS EXISTS:
//   The service layer owns business logic — rules that are independent of HTTP or SQL.
//
//   Example rule: "A product can only be created if its price is greater than zero."
//   This rule doesn't belong in the handler (HTTP concerns) or the repository (SQL concerns).
//   It lives here.
//
//   At V1 the service layer is thin. As the system grows, it will contain:
//     - Transactional workflows (create order → decrement inventory → create payment)
//     - Cross-entity rules (a flash sale cannot start in the past)
//     - Integration points (emit event to Kafka after order is placed — V6)
//
// RUST CONCEPT — Why we pass `&PgPool` down through layers:
//   `PgPool` is an Arc<Inner> internally — it's cheap to clone and share.
//   Passing a reference (&PgPool) avoids even the clone cost.
//   The pool is owned by AppState and lives for the entire program lifetime.

use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::product::{CreateProductRequest, Product},
    repositories::{cache_repository, product_repository},
};

pub async fn get_product(pool: &PgPool, redis_pool: &RedisPool, id: Uuid) -> AppResult<Product> {
    // 1. Try Redis Cache
    if let Ok(Some(cached_product)) = cache_repository::get_product(redis_pool, id).await {
        return Ok(cached_product);
    }

    // 2. Cache Miss -> Query PostgreSQL Database
    let product = product_repository::get_product(pool, id)
        .await?
        .ok_or(AppError::NotFound)?;

    // 3. Populate Redis Cache
    let _ = cache_repository::set_product(redis_pool, &product).await;

    Ok(product)
}

pub async fn list_products(
    pool: &PgPool,
    redis_pool: &RedisPool,
    limit: i64,
    offset: i64,
) -> AppResult<Vec<Product>> {
    let limit = limit.clamp(1, 100); // Never allow unbounded queries from clients
    let offset = offset.max(0);

    // 1. Try Redis Cache
    if let Ok(Some(cached_products)) = cache_repository::get_products_list(redis_pool, limit, offset).await {
        return Ok(cached_products);
    }

    // 2. Cache Miss -> Query PostgreSQL Database
    let products = product_repository::list_products(pool, limit, offset).await?;

    // 3. Populate Redis Cache
    let _ = cache_repository::set_products_list(redis_pool, limit, offset, &products).await;

    Ok(products)
}

pub async fn create_product(
    pool: &PgPool,
    redis_pool: &RedisPool,
    req: CreateProductRequest,
) -> AppResult<Product> {
    // Business rule validation
    req.validate().map_err(AppError::Validation)?;

    let product = product_repository::create_product(pool, &req).await?;

    // Set product cache
    let _ = cache_repository::set_product(redis_pool, &product).await;

    Ok(product)
}

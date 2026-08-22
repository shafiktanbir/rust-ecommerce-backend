// src/repositories/cache_repository.rs

use deadpool_redis::{redis::AsyncCommands, Pool};
use uuid::Uuid;

use crate::{errors::AppResult, models::product::Product};

const PRODUCT_CACHE_TTL_SECS: u64 = 60; // 60 seconds TTL

pub async fn get_product(redis_pool: &Pool, id: Uuid) -> AppResult<Option<Product>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool connection error: {:?}", e);
            return Ok(None);
        }
    };

    let key = format!("product:{}", id);
    let val: Option<String> = match conn.get(&key).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Redis GET error for key {}: {:?}", key, e);
            return Ok(None);
        }
    };

    if let Some(json_str) = val {
        if let Ok(product) = serde_json::from_str::<Product>(&json_str) {
            tracing::debug!("CACHE HIT: product:{}", id);
            return Ok(Some(product));
        }
    }

    tracing::debug!("CACHE MISS: product:{}", id);
    Ok(None)
}

pub async fn set_product(redis_pool: &Pool, product: &Product) -> AppResult<()> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool connection error: {:?}", e);
            return Ok(());
        }
    };

    let key = format!("product:{}", product.id);
    if let Ok(json_str) = serde_json::to_string(product) {
        let _: Result<(), _> = conn.set_ex(&key, json_str, PRODUCT_CACHE_TTL_SECS).await;
        tracing::debug!(
            "CACHE SET: product:{} (TTL={}s)",
            product.id,
            PRODUCT_CACHE_TTL_SECS
        );
    }

    Ok(())
}

pub async fn invalidate_product(redis_pool: &Pool, id: Uuid) -> AppResult<()> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool connection error: {:?}", e);
            return Ok(());
        }
    };

    let key = format!("product:{}", id);
    let _: Result<(), _> = conn.del(&key).await;
    tracing::debug!("CACHE INVALIDATED: product:{}", id);

    Ok(())
}

pub async fn get_products_list(
    redis_pool: &Pool,
    limit: i64,
    offset: i64,
) -> AppResult<Option<Vec<Product>>> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool connection error: {:?}", e);
            return Ok(None);
        }
    };

    let key = format!("products:list:{}:{}", limit, offset);
    let val: Option<String> = match conn.get(&key).await {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Redis GET error for key {}: {:?}", key, e);
            return Ok(None);
        }
    };

    if let Some(json_str) = val {
        if let Ok(products) = serde_json::from_str::<Vec<Product>>(&json_str) {
            tracing::debug!("CACHE HIT: products:list:{}:{}", limit, offset);
            return Ok(Some(products));
        }
    }

    tracing::debug!("CACHE MISS: products:list:{}:{}", limit, offset);
    Ok(None)
}

pub async fn set_products_list(
    redis_pool: &Pool,
    limit: i64,
    offset: i64,
    products: &[Product],
) -> AppResult<()> {
    let mut conn = match redis_pool.get().await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Redis pool connection error: {:?}", e);
            return Ok(());
        }
    };

    let key = format!("products:list:{}:{}", limit, offset);
    if let Ok(json_str) = serde_json::to_string(products) {
        let _: Result<(), _> = conn.set_ex(&key, json_str, PRODUCT_CACHE_TTL_SECS).await;
        tracing::debug!(
            "CACHE SET: products:list:{}:{} (TTL={}s)",
            limit,
            offset,
            PRODUCT_CACHE_TTL_SECS
        );
    }

    Ok(())
}

// src/services/order_service.rs

use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    jobs::{
        queue::enqueue_job,
        types::{Job, JobPayload},
    },
    models::order::{CreateOrderDto, Order, OrderResponseDto},
    repositories::order_repository,
};

pub async fn create_order(
    pool: &PgPool,
    redis_pool: &RedisPool,
    user_id: Uuid,
    dto: CreateOrderDto,
) -> AppResult<OrderResponseDto> {
    if dto.items.is_empty() {
        return Err(AppError::Validation(
            "Order must contain at least one item".into(),
        ));
    }

    // 1. Fast atomic database transaction (inventory decrement + order insert)
    let order = order_repository::create_order(pool, user_id, &dto.items).await?;

    // 2. Decoupled Asynchronous Job Enqueueing (< 2ms execution budget)
    let email_job = Job::new(JobPayload::SendOrderConfirmationEmail {
        order_id: order.id,
        user_id: order.user_id,
        total_amount: order.total_amount,
    });

    let invoice_job = Job::new(JobPayload::GenerateInvoice {
        order_id: order.id,
        user_id: order.user_id,
    });

    if let Err(e) = enqueue_job(redis_pool, &email_job).await {
        tracing::error!("Failed to enqueue email job for order {}: {:?}", order.id, e);
    }

    if let Err(e) = enqueue_job(redis_pool, &invoice_job).await {
        tracing::error!("Failed to enqueue invoice job for order {}: {:?}", order.id, e);
    }

    info!("Order ID {} created instantly and 2 background jobs enqueued", order.id);

    Ok(order)
}

pub async fn list_user_orders(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Order>> {
    order_repository::list_orders_by_user(pool, user_id).await
}

pub async fn get_order(
    pool: &PgPool,
    order_id: Uuid,
    user_id: Uuid,
) -> AppResult<OrderResponseDto> {
    order_repository::get_order_by_id(pool, order_id, user_id).await
}

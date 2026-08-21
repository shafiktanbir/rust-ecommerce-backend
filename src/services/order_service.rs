// src/services/order_service.rs

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::order::{CreateOrderDto, Order, OrderResponseDto},
    repositories::order_repository,
};

pub async fn create_order(
    pool: &PgPool,
    user_id: Uuid,
    dto: CreateOrderDto,
) -> AppResult<OrderResponseDto> {
    if dto.items.is_empty() {
        return Err(AppError::Validation("Order must contain at least one item".into()));
    }

    order_repository::create_order(pool, user_id, &dto.items).await
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

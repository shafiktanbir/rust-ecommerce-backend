// src/handlers/orders.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::order::{CreateOrderDto, Order, OrderResponseDto},
    routes::AppState,
    services::order_service,
};

pub async fn create_order(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(payload): Json<CreateOrderDto>,
) -> AppResult<(StatusCode, Json<OrderResponseDto>)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))?;

    let order = order_service::create_order(&state.db.writer, &state.redis, user_id, payload).await?;
    
    // 🛡️ Read-Your-Own-Writes Consistency: Mark user sticky to Primary DB for 5 seconds after order checkout
    state.set_user_sticky_primary(&claims.sub, 5).await;

    Ok((StatusCode::CREATED, Json(order)))
}

pub async fn list_orders(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> AppResult<Json<Vec<Order>>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))?;

    // 🛡️ Sticky-aware pool selection: routes to Primary if user created an order within last 5 seconds
    let pool = state.get_reader_pool_for_user(Some(&claims.sub)).await;
    let orders = order_service::list_user_orders(pool, user_id).await?;
    Ok(Json(orders))
}

pub async fn get_order(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<OrderResponseDto>> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid user ID in token".into()))?;

    let pool = state.get_reader_pool_for_user(Some(&claims.sub)).await;
    let order = order_service::get_order(pool, order_id, user_id).await?;
    Ok(Json(order))
}

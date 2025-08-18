// src/models/order.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderItemDto {
    pub product_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderDto {
    pub items: Vec<CreateOrderItemDto>,
}

#[derive(Debug, Serialize)]
pub struct OrderResponseDto {
    pub id: Uuid,
    pub user_id: Uuid,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub items: Vec<OrderItemResponseDto>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct OrderItemResponseDto {
    pub id: Uuid,
    pub product_id: Uuid,
    pub quantity: i32,
    pub unit_price: f64,
}

impl CreateOrderDto {
    pub fn calculate_subtotal(items: &[(i32, f64)]) -> f64 {
        items.iter().map(|(qty, price)| (*qty as f64) * price).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_subtotal_calculation() {
        let items = vec![(2, 29.99), (1, 49.99), (3, 10.00)];
        let total = CreateOrderDto::calculate_subtotal(&items);
        assert!((total - 139.97).abs() < 1e-6);
    }

    #[test]
    fn test_order_status_variants() {
        assert_eq!(OrderStatus::Pending, OrderStatus::Pending);
        assert_ne!(OrderStatus::Pending, OrderStatus::Confirmed);
    }
}

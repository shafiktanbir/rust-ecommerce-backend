// src/repositories/order_repository.rs

use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::{
        order::{CreateOrderItemDto, Order, OrderItem, OrderItemResponseDto, OrderResponseDto},
        product::Product,
    },
};

pub async fn create_order(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateOrderItemDto],
) -> AppResult<OrderResponseDto> {
    if items.is_empty() {
        return Err(AppError::Validation("Order must contain at least one item".into()));
    }

    let mut tx = pool.begin().await?;

    let mut total_amount: f64 = 0.0;
    let mut item_responses = Vec::new();
    let mut product_updates = Vec::new();

    for item in items {
        if item.quantity <= 0 {
            return Err(AppError::Validation("Quantity must be greater than zero".into()));
        }

        // Lock product row FOR UPDATE to prevent race conditions during high concurrency
        let product = sqlx::query_as::<_, Product>(
            r#"
            SELECT id, name, description, price, inventory_count, created_at, updated_at
            FROM products
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(item.product_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| AppError::NotFound)?;

        if product.inventory_count < item.quantity {
            return Err(AppError::Validation(format!(
                "Insufficient inventory for product '{}'. Requested: {}, Available: {}",
                product.name, item.quantity, product.inventory_count
            )));
        }

        let item_total = product.price * (item.quantity as f64);
        total_amount += item_total;

        product_updates.push((product.id, item.quantity, product.price));
    }

    // Insert order
    let order = sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO orders (user_id, total_amount, status)
        VALUES ($1, $2, 'pending'::order_status)
        RETURNING id, user_id, total_amount::FLOAT8 AS total_amount, status, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(total_amount)
    .fetch_one(&mut *tx)
    .await?;

    // Decrement inventory and insert order items
    for (product_id, quantity, unit_price) in product_updates {
        sqlx::query(
            r#"
            UPDATE products
            SET inventory_count = inventory_count - $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(quantity)
        .bind(product_id)
        .execute(&mut *tx)
        .await?;

        let order_item = sqlx::query_as::<_, OrderItem>(
            r#"
            INSERT INTO order_items (order_id, product_id, quantity, unit_price)
            VALUES ($1, $2, $3, $4)
            RETURNING id, order_id, product_id, quantity, unit_price::FLOAT8 AS unit_price, created_at
            "#,
        )
        .bind(order.id)
        .bind(product_id)
        .bind(quantity)
        .bind(unit_price)
        .fetch_one(&mut *tx)
        .await?;

        item_responses.push(OrderItemResponseDto {
            id: order_item.id,
            product_id: order_item.product_id,
            quantity: order_item.quantity,
            unit_price: order_item.unit_price,
        });
    }

    tx.commit().await?;

    Ok(OrderResponseDto {
        id: order.id,
        user_id: order.user_id,
        total_amount: order.total_amount,
        status: order.status,
        items: item_responses,
        created_at: order.created_at,
    })
}

pub async fn list_orders_by_user(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Order>> {
    let orders = sqlx::query_as::<_, Order>(
        r#"
        SELECT id, user_id, total_amount::FLOAT8 AS total_amount, status, created_at, updated_at
        FROM orders
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(orders)
}

pub async fn get_order_by_id(pool: &PgPool, order_id: Uuid, user_id: Uuid) -> AppResult<OrderResponseDto> {
    let order = sqlx::query_as::<_, Order>(
        r#"
        SELECT id, user_id, total_amount::FLOAT8 AS total_amount, status, created_at, updated_at
        FROM orders
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(order_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    let items = sqlx::query_as::<_, OrderItem>(
        r#"
        SELECT id, order_id, product_id, quantity, unit_price::FLOAT8 AS unit_price, created_at
        FROM order_items
        WHERE order_id = $1
        "#,
    )
    .bind(order.id)
    .fetch_all(pool)
    .await?;

    let item_responses = items
        .into_iter()
        .map(|i| OrderItemResponseDto {
            id: i.id,
            product_id: i.product_id,
            quantity: i.quantity,
            unit_price: i.unit_price,
        })
        .collect();

    Ok(OrderResponseDto {
        id: order.id,
        user_id: order.user_id,
        total_amount: order.total_amount,
        status: order.status,
        items: item_responses,
        created_at: order.created_at,
    })
}

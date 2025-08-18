// src/models/product.rs
//
// WHY THIS EXISTS:
//   Models are pure data structures — they represent what exists in the database
//   and what travels over the wire as JSON.
//
//   Keeping models separate from business logic means:
//     - Easy to serialize/deserialize with serde
//     - Clear contract between layers (handler ↔ service ↔ repository)
//     - Can evolve API shape independently from DB schema if needed
//
// RUST CONCEPT — derive macros:
//   `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]`
//   Each of these generates code at compile time:
//     - Debug:      allows {:?} printing
//     - Clone:      allows .clone() for passing copies
//     - Serialize:  converts to JSON (for responses)
//     - Deserialize: parses from JSON (for requests)
//     - FromRow:    sqlx can map a PostgreSQL row → Product struct automatically

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a product row from the `products` table.
/// Used in API responses.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    /// Stored as NUMERIC in PostgreSQL; mapped to rust_decimal or f64.
    /// For simplicity in V1 we use f64. In production use rust_decimal to avoid floating-point errors.
    pub price: f64,
    /// How many units are available. Critical for flash-sale concurrency.
    pub inventory_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for creating a new product.
/// Note: id, created_at, updated_at are generated server-side — not accepted from clients.
#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub price: f64,
    pub inventory_count: i32,
}

impl CreateProductRequest {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Product name cannot be empty".to_string());
        }
        if self.price < 0.0 {
            return Err("Price cannot be negative".to_string());
        }
        if self.inventory_count < 0 {
            return Err("Inventory count cannot be negative".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_product_request() {
        let req = CreateProductRequest {
            name: "High-Performance Rust Server".to_string(),
            description: Some("Built with Axum".to_string()),
            price: 99.99,
            inventory_count: 50,
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_invalid_empty_name() {
        let req = CreateProductRequest {
            name: "   ".to_string(),
            description: None,
            price: 49.99,
            inventory_count: 10,
        };
        assert_eq!(req.validate().unwrap_err(), "Product name cannot be empty");
    }

    #[test]
    fn test_invalid_negative_price() {
        let req = CreateProductRequest {
            name: "Server".to_string(),
            description: None,
            price: -10.0,
            inventory_count: 10,
        };
        assert_eq!(req.validate().unwrap_err(), "Price cannot be negative");
    }

    #[test]
    fn test_invalid_negative_inventory() {
        let req = CreateProductRequest {
            name: "Server".to_string(),
            description: None,
            price: 10.0,
            inventory_count: -5,
        };
        assert_eq!(req.validate().unwrap_err(), "Inventory count cannot be negative");
    }
}

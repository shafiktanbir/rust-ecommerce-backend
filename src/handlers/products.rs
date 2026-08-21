// src/handlers/products.rs
//
// WHY THIS EXISTS:
//   Handlers are the HTTP boundary layer. They:
//     1. Extract data from the HTTP request (path params, query params, JSON body)
//     2. Call the service layer
//     3. Convert the result into an HTTP response
//
//   Handlers should contain NO business logic and NO SQL.
//   If a handler is doing anything complex, that logic belongs in a service.
//
// RUST CONCEPT — State<AppState>:
//   `State<AppState>` is an Axum extractor. When Axum sees this in a handler signature,
//   it clones the AppState from the router and injects it.
//   AppState holds the PgPool (wrapped in Arc internally), so cloning is cheap.
//
// RUST CONCEPT — Json<T>:
//   `Json<CreateProductRequest>` automatically:
//     1. Reads the request body
//     2. Deserializes it from JSON into CreateProductRequest using serde
//     3. Returns 422 with an error if deserialization fails
//   At the response side, `Json(product)` serializes the struct to JSON and sets Content-Type.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    errors::AppResult, models::product::CreateProductRequest, routes::AppState,
    services::product_service,
};

/// Query params for listing products
#[derive(Debug, Deserialize)]
pub struct ListProductsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    20
}

/// GET /products?limit=20&offset=0
pub async fn list_products(
    State(state): State<AppState>,
    Query(params): Query<ListProductsQuery>,
) -> AppResult<impl IntoResponse> {
    let products =
        product_service::list_products(&state.db, &state.redis, params.limit, params.offset).await?;
    Ok((StatusCode::OK, Json(products)))
}

/// GET /products/:id
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let product = product_service::get_product(&state.db, &state.redis, id).await?;
    Ok((StatusCode::OK, Json(product)))
}

/// POST /products
/// Body: { "name": "...", "price": 9.99, "inventory_count": 100 }
pub async fn create_product(
    State(state): State<AppState>,
    Json(req): Json<CreateProductRequest>,
) -> AppResult<impl IntoResponse> {
    let product = product_service::create_product(&state.db, &state.redis, req).await?;
    Ok((StatusCode::CREATED, Json(product)))
}

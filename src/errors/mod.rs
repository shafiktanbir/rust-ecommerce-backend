// src/errors/mod.rs
//
// WHY THIS EXISTS:
//   In a real API, many things can go wrong:
//     - Database row not found
//     - SQL query fails
//     - Invalid user input
//     - Unexpected internal errors
//
//   Without a central error type, every handler has to manually convert errors into
//   HTTP responses — duplicating logic and making it easy to accidentally leak
//   internal error details (SQL messages, stack traces) to clients.
//
//   AppError is the single type that:
//     1. Describes all possible error categories
//     2. Converts them to appropriate HTTP status codes
//     3. Keeps internal details server-side (not exposed to clients)
//
// RUST CONCEPT — thiserror:
//   `thiserror` generates `Display` and `Error` implementations from derive macros.
//   This keeps error definition concise while maintaining full Rust error trait support.
//
// RUST CONCEPT — IntoResponse (axum):
//   Axum requires handlers to return types that implement `IntoResponse`.
//   By implementing `IntoResponse` for `AppError`, we can use `?` operator in handlers
//   and let errors propagate cleanly up to the HTTP layer.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Resource not found")]
    NotFound,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // IMPORTANT: We log the internal error but return a clean message to the client.
        // Never expose raw SQL errors or stack traces to external callers — this leaks
        // schema information and is a security risk.
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found".to_string()),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.clone()),
            AppError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "A database error occurred".to_string(),
                )
            }
            AppError::Internal(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal error occurred".to_string(),
                )
            }
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

/// Convenient type alias — handlers return `Result<T, AppError>` throughout the codebase
pub type AppResult<T> = Result<T, AppError>;

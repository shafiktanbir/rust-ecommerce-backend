// src/middleware/auth.rs

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};

use crate::{
    errors::AppError,
    models::user::Claims,
    routes::AppState,
    services::auth_service,
};

/// Extractor for authenticated user claims.
/// Use `auth_user: AuthUser` in handlers to protect routes.
#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|val| val.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".into()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized("Invalid Authorization header format".into()));
        }

        let token = &auth_header[7..];

        let claims = auth_service::verify_token(token, &state.config.jwt_secret)?;

        Ok(AuthUser(claims))
    }
}

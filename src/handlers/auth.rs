// src/handlers/auth.rs

use axum::{extract::State, http::StatusCode, Json};

use crate::{
    errors::AppResult,
    models::user::{AuthResponseDto, LoginUserDto, RegisterUserDto},
    routes::AppState,
    services::auth_service,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserDto>,
) -> AppResult<(StatusCode, Json<AuthResponseDto>)> {
    let result = auth_service::register(&state.db.writer, &state.config.jwt_secret, payload).await?;
    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginUserDto>,
) -> AppResult<Json<AuthResponseDto>> {
    let result = auth_service::login(state.get_reader_pool(), &state.config.jwt_secret, payload).await?;
    Ok(Json(result))
}

// src/handlers/queue_stats.rs

use axum::{extract::State, Json};

use crate::{
    errors::AppResult,
    jobs::{queue::get_queue_stats, types::QueueStats},
    routes::AppState,
};

pub async fn get_stats(
    State(state): State<AppState>,
) -> AppResult<Json<QueueStats>> {
    let stats = get_queue_stats(&state.redis).await?;
    Ok(Json(stats))
}

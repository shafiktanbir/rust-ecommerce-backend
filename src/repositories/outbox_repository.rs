// src/repositories/outbox_repository.rs

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::outbox::OutboxRecord,
};

pub async fn insert_outbox_event_tx(
    tx: &mut Transaction<'_, Postgres>,
    aggregate_type: &str,
    aggregate_id: Uuid,
    event_type: &str,
    payload: serde_json::Value,
) -> AppResult<Uuid> {
    let outbox_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO outbox (id, aggregate_type, aggregate_id, event_type, payload, status, created_at)
        VALUES ($1, $2, $3, $4, $5, 'pending', NOW())
        "#,
        outbox_id,
        aggregate_type,
        aggregate_id,
        event_type,
        payload
    )
    .execute(&mut **tx)
    .await
    .map_err(AppError::Database)?;

    Ok(outbox_id)
}

pub async fn fetch_pending_outbox_events(
    pool: &PgPool,
    limit: i64,
) -> AppResult<Vec<OutboxRecord>> {
    let records = sqlx::query_as!(
        OutboxRecord,
        r#"
        SELECT id, aggregate_type, aggregate_id, event_type, payload, status, retry_count, created_at, processed_at
        FROM outbox
        WHERE status = 'pending' AND retry_count < 5
        ORDER BY created_at ASC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(records)
}

pub async fn mark_outbox_processed(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE outbox
        SET status = 'processed', processed_at = NOW()
        WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(())
}

pub async fn increment_outbox_retry(pool: &PgPool, id: Uuid) -> AppResult<()> {
    sqlx::query!(
        r#"
        UPDATE outbox
        SET retry_count = retry_count + 1
        WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await
    .map_err(AppError::Database)?;

    Ok(())
}

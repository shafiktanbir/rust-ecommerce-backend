// src/jobs/queue.rs

use deadpool_redis::{redis::AsyncCommands, Pool};
use tracing::{error, info};

use crate::{
    errors::{AppError, AppResult},
    jobs::types::{Job, QueueStats},
};

const QUEUE_PENDING: &str = "ecommerce_job_queue";
const QUEUE_PROCESSING: &str = "ecommerce_job_processing";
const QUEUE_DLQ: &str = "ecommerce_job_dlq";
const STATS_PROCESSED: &str = "ecommerce_stats_processed";
const STATS_FAILED: &str = "ecommerce_stats_failed";

/// Enqueue a new background job into Redis using LPUSH.
pub async fn enqueue_job(pool: &Pool, job: &Job) -> AppResult<()> {
    let mut conn = pool.get().await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to acquire Redis conn for queue: {}", e))
    })?;

    let payload_json = serde_json::to_string(job).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to serialize job payload: {}", e))
    })?;

    conn.lpush::<_, _, ()>(QUEUE_PENDING, payload_json)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis LPUSH failed: {}", e)))?;

    Ok(())
}

/// Dequeue a job atomically from QUEUE_PENDING using RPOPLPUSH (or RPOP if temporary).
pub async fn dequeue_job(pool: &Pool) -> AppResult<Option<Job>> {
    let mut conn = pool.get().await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to acquire Redis conn for dequeue: {}", e))
    })?;

    let raw_job: Option<String> = conn
        .rpoplpush(QUEUE_PENDING, QUEUE_PROCESSING)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis RPOPLPUSH failed: {}", e)))?;

    match raw_job {
        Some(json_str) => match serde_json::from_str::<Job>(&json_str) {
            Ok(job) => Ok(Some(job)),
            Err(e) => {
                error!("Failed to deserialize job from Redis: {}", e);
                Ok(None)
            }
        },
        None => Ok(None),
    }
}

/// Acknowledge job completion: remove from processing list and increment processed counter.
pub async fn ack_job(pool: &Pool, job: &Job) -> AppResult<()> {
    let mut conn = pool.get().await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to acquire Redis conn for ack: {}", e))
    })?;

    let payload_json = serde_json::to_string(job).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to serialize job payload: {}", e))
    })?;

    let _: () = conn
        .lrem(QUEUE_PROCESSING, 1, &payload_json)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis LREM failed: {}", e)))?;

    let _: () = conn
        .incr(STATS_PROCESSED, 1)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Redis INCR failed: {}", e)))?;

    Ok(())
}

/// Handle job failure: retry with exponential backoff or move to DLQ if max retries exceeded.
pub async fn fail_job(pool: &Pool, mut job: Job) -> AppResult<()> {
    let mut conn = pool.get().await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to acquire Redis conn for fail_job: {}", e))
    })?;

    let old_payload_json = serde_json::to_string(&job).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to serialize job payload: {}", e))
    })?;

    // Remove from processing queue
    let _: () = conn
        .lrem(QUEUE_PROCESSING, 1, &old_payload_json)
        .await
        .unwrap_or(());

    job.attempts += 1;

    if job.attempts >= job.max_retries {
        error!(
            "Job ID {} exceeded max retries ({}/{}). Pushing to Dead Letter Queue (DLQ)",
            job.id, job.attempts, job.max_retries
        );
        let dlq_json = serde_json::to_string(&job).unwrap_or(old_payload_json);
        let _: () = conn.lpush(QUEUE_DLQ, dlq_json).await.unwrap_or(());
        let _: () = conn.incr(STATS_FAILED, 1).await.unwrap_or(());
    } else {
        info!("Re-enqueueing job ID {} (attempt {})", job.id, job.attempts);
        let retry_json = serde_json::to_string(&job).unwrap_or(old_payload_json);
        let _: () = conn.lpush(QUEUE_PENDING, retry_json).await.unwrap_or(());
    }

    Ok(())
}

/// Fetch current stats of the job queue.
pub async fn get_queue_stats(pool: &Pool) -> AppResult<QueueStats> {
    let mut conn = pool.get().await.map_err(|e| {
        AppError::Internal(anyhow::anyhow!("Failed to acquire Redis conn for stats: {}", e))
    })?;

    let pending: i64 = conn.llen(QUEUE_PENDING).await.unwrap_or(0);
    let processing: i64 = conn.llen(QUEUE_PROCESSING).await.unwrap_or(0);
    let dlq: i64 = conn.llen(QUEUE_DLQ).await.unwrap_or(0);

    Ok(QueueStats {
        pending_jobs: pending,
        processing_jobs: processing,
        failed_jobs: dlq,
    })
}

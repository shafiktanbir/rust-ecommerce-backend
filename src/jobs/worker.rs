// src/jobs/worker.rs

use deadpool_redis::Pool;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

use crate::jobs::{
    queue,
    types::{Job, JobPayload},
};

/// Start the background worker loop.
/// Spawns worker tasks based on `worker_count`.
pub fn start_worker_pool(redis_pool: Pool, worker_count: usize) {
    info!("Starting {} background job queue workers...", worker_count);

    for worker_id in 1..=worker_count {
        let pool = redis_pool.clone();
        tokio::spawn(async move {
            info!("Worker #{} online and polling queue", worker_id);
            loop {
                match queue::dequeue_job(&pool).await {
                    Ok(Some(job)) => {
                        info!("Worker #{} processing job ID: {}", worker_id, job.id);
                        let start = std::time::Instant::now();
                        
                        if let Err(err) = process_job(&job).await {
                            error!("Worker #{} failed job ID {}: {:?}", worker_id, job.id, err);
                            let _ = queue::fail_job(&pool, job).await;
                        } else {
                            let elapsed = start.elapsed();
                            info!(
                                "Worker #{} completed job ID {} in {:?}",
                                worker_id, job.id, elapsed
                            );
                            let _ = queue::ack_job(&pool, &job).await;
                        }
                    }
                    Ok(None) => {
                        // Queue empty: sleep 10ms to prevent idle CPU spinning
                        sleep(Duration::from_millis(10)).await;
                    }
                    Err(e) => {
                        error!("Worker #{} error polling queue: {:?}", worker_id, e);
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });
    }
}

/// Process specific job payload asynchronously.
async fn process_job(job: &Job) -> Result<(), String> {
    match &job.payload {
        JobPayload::SendOrderConfirmationEmail {
            order_id,
            user_id,
            total_amount,
        } => {
            info!(
                "Processing async email confirmation for Order ID {} (User: {}, Total: ${:.2})",
                order_id, user_id, total_amount
            );
            // Simulated 200ms SMTP email sending delay in background worker
            sleep(Duration::from_millis(200)).await;
            Ok(())
        }
        JobPayload::GenerateInvoice { order_id, user_id } => {
            info!(
                "Processing async invoice PDF generation for Order ID {} (User: {})",
                order_id, user_id
            );
            // Simulated 50ms PDF generation delay in background worker
            sleep(Duration::from_millis(50)).await;
            Ok(())
        }
        JobPayload::SyncAnalyticsEvent { event_name, order_id } => {
            info!(
                "Syncing analytics event '{}' for Order ID {}",
                event_name, order_id
            );
            sleep(Duration::from_millis(10)).await;
            Ok(())
        }
    }
}

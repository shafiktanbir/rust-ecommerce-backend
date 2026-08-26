// src/jobs/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobPayload {
    SendOrderConfirmationEmail {
        order_id: Uuid,
        user_id: Uuid,
        total_amount: f64,
    },
    GenerateInvoice {
        order_id: Uuid,
        user_id: Uuid,
    },
    SyncAnalyticsEvent {
        event_name: String,
        order_id: Uuid,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub payload: JobPayload,
    pub attempts: u32,
    pub max_retries: u32,
    pub created_at: DateTime<Utc>,
}

impl Job {
    pub fn new(payload: JobPayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            payload,
            attempts: 0,
            max_retries: 3,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending_jobs: i64,
    pub processing_jobs: i64,
    pub failed_jobs: i64,
}

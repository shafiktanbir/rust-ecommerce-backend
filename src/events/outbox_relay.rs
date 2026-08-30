// src/events/outbox_relay.rs

use std::time::Duration;
use sqlx::PgPool;
use tracing::{error, info};

use crate::{
    events::producer::KafkaProducer,
    repositories::outbox_repository,
};

pub fn start_outbox_relay(pool: PgPool, producer: KafkaProducer) {
    tokio::spawn(async move {
        info!("Starting Outbox Relay Worker background task...");
        let mut interval = tokio::time::interval(Duration::from_millis(500));

        loop {
            interval.tick().await;

            match outbox_repository::fetch_pending_outbox_events(&pool, 50).await {
                Ok(records) => {
                    if records.is_empty() {
                        continue;
                    }

                    info!("Outbox Relay found {} pending outbox records to process", records.len());

                    for record in records {
                        let topic = format!("ecom-{}-events", record.aggregate_type.to_lowercase());
                        let key = record.aggregate_id.to_string();
                        let payload_bytes = match serde_json::to_vec(&record.payload) {
                            Ok(b) => b,
                            Err(e) => {
                                error!("Failed to serialize outbox payload for {}: {:?}", record.id, e);
                                continue;
                            }
                        };

                        match producer.send_event(&topic, &key, &payload_bytes).await {
                            Ok(_) => {
                                if let Err(e) = outbox_repository::mark_outbox_processed(&pool, record.id).await {
                                    error!("Failed to mark outbox record {} as processed: {:?}", record.id, e);
                                } else {
                                    info!("Outbox event {} processed & marked DONE in PostgreSQL", record.id);
                                }
                            }
                            Err(e) => {
                                error!("Kafka Relay failed for outbox event {}: {}", record.id, e);
                                if let Err(db_err) = outbox_repository::increment_outbox_retry(&pool, record.id).await {
                                    error!("Failed to increment retry count for outbox record {}: {:?}", record.id, db_err);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Outbox Relay error fetching pending records: {:?}", e);
                }
            }
        }
    });
}

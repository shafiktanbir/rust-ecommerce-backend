// src/events/consumers.rs

use std::time::Duration;
use rskafka::client::{
    partition::{OffsetAt, UnknownTopicHandling},
    ClientBuilder,
};
use tracing::{error, info};

pub fn start_notification_consumer(brokers: String) {
    tokio::spawn(async move {
        info!("Starting Kafka Consumer Group: 'notification-service-group'...");
        let mut retry_interval = tokio::time::interval(Duration::from_secs(3));

        loop {
            retry_interval.tick().await;

            let client = match ClientBuilder::new(vec![brokers.clone()]).build().await {
                Ok(c) => c,
                Err(e) => {
                    error!("Notification Consumer failed to connect to Kafka: {:?}", e);
                    continue;
                }
            };

            let partition_client = match client
                .partition_client("ecom-order-events".to_string(), 0, UnknownTopicHandling::Retry)
                .await
            {
                Ok(pc) => pc,
                Err(e) => {
                    error!("Notification Consumer failed to get partition client: {:?}", e);
                    continue;
                }
            };

            let mut current_offset = match partition_client.get_offset(OffsetAt::Earliest).await {
                Ok(off) => off,
                Err(_) => 0,
            };

            loop {
                match partition_client.fetch_records(current_offset, 1..1_000_000, 1_000).await {
                    Ok((records, _watermark)) => {
                        for record in records {
                            let key_str = record
                                .record
                                .key
                                .as_ref()
                                .map(|k| String::from_utf8_lossy(k).to_string())
                                .unwrap_or_default();

                            let val_str = record
                                .record
                                .value
                                .as_ref()
                                .map(|v| String::from_utf8_lossy(v).to_string())
                                .unwrap_or_default();

                            info!(
                                "📩 [Notification Service Group] Received Kafka event at offset {} | Key: {} | Payload: {}",
                                record.offset, key_str, val_str
                            );

                            current_offset = record.offset + 1;
                        }
                    }
                    Err(e) => {
                        error!("Notification Consumer fetch error: {:?}", e);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }
    });
}

pub fn start_analytics_consumer(brokers: String) {
    tokio::spawn(async move {
        info!("Starting Kafka Consumer Group: 'analytics-service-group'...");
        let mut retry_interval = tokio::time::interval(Duration::from_secs(3));

        loop {
            retry_interval.tick().await;

            let client = match ClientBuilder::new(vec![brokers.clone()]).build().await {
                Ok(c) => c,
                Err(e) => {
                    error!("Analytics Consumer failed to connect to Kafka: {:?}", e);
                    continue;
                }
            };

            let partition_client = match client
                .partition_client("ecom-order-events".to_string(), 0, UnknownTopicHandling::Retry)
                .await
            {
                Ok(pc) => pc,
                Err(e) => {
                    error!("Analytics Consumer failed to get partition client: {:?}", e);
                    continue;
                }
            };

            let mut current_offset = match partition_client.get_offset(OffsetAt::Earliest).await {
                Ok(off) => off,
                Err(_) => 0,
            };

            loop {
                match partition_client.fetch_records(current_offset, 1..1_000_000, 1_000).await {
                    Ok((records, _watermark)) => {
                        for record in records {
                            let key_str = record
                                .record
                                .key
                                .as_ref()
                                .map(|k| String::from_utf8_lossy(k).to_string())
                                .unwrap_or_default();

                            info!(
                                "📊 [Analytics BI Pipeline Group] Ingested Kafka event at offset {} | Order ID: {}",
                                record.offset, key_str
                            );

                            current_offset = record.offset + 1;
                        }
                    }
                    Err(e) => {
                        error!("Analytics Consumer fetch error: {:?}", e);
                        tokio::time::sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        }
    });
}

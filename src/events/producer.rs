// src/events/producer.rs

use chrono::Utc;
use rskafka::{
    client::{
        partition::{Compression, UnknownTopicHandling},
        ClientBuilder,
    },
    record::Record,
};
use tracing::info;

#[derive(Clone)]
pub struct KafkaProducer {
    brokers: String,
}

impl KafkaProducer {
    pub fn new(brokers: String) -> Self {
        Self { brokers }
    }

    pub async fn send_event(&self, topic: &str, key: &str, payload: &[u8]) -> Result<(), String> {
        let connection_str = self.brokers.clone();
        let client = ClientBuilder::new(vec![connection_str])
            .build()
            .await
            .map_err(|e| format!("Failed to build Kafka client: {:?}", e))?;

        let partition_client = client
            .partition_client(topic.to_string(), 0, UnknownTopicHandling::Retry)
            .await
            .map_err(|e| format!("Failed to get partition client for topic '{}': {:?}", topic, e))?;



        let record = Record {
            key: Some(key.as_bytes().to_vec()),
            value: Some(payload.to_vec()),
            headers: Default::default(),
            timestamp: Utc::now(),
        };

        partition_client
            .produce(vec![record], Compression::NoCompression)
            .await
            .map_err(|e| format!("Failed to produce Kafka record: {:?}", e))?;

        info!(
            "Published event to Kafka topic '{}' with key '{}'",
            topic, key
        );
        Ok(())
    }
}

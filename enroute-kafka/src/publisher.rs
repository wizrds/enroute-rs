use std::time::Duration;
use chrono::Utc;
use async_trait::async_trait;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    message::{OwnedHeaders, Header},
};

use enroute_core::{
    event::Event,
    error::{Error, Result},
    publisher::Publisher,
};


#[derive(Clone)]
pub struct KafkaPublisher {
    producer: FutureProducer,
    topic: String,
    timeout: Duration,
}

impl KafkaPublisher {
    pub fn new(producer: FutureProducer, topic: String, timeout: Option<Duration>) -> Self {
        Self {
            producer,
            topic,
            timeout: timeout.unwrap_or_else(|| Duration::from_secs(0)),
        }
    }

    pub async fn publish(&self, event: Event) -> Result<()> {
        let event_id = event.id().to_string();
        let payload = event.data_as_bytes()?;
        let record = FutureRecord::<'_, String, Vec<u8>>::to(&self.topic)
            .key(&event_id)
            .timestamp(
                event
                    .time()
                    .map(|t| t.timestamp_millis())
                    .unwrap_or_else(|| Utc::now().timestamp_millis())
            )
            .headers(
                OwnedHeaders::new()
                    .insert(Header {
                        key: "ce-specversion",
                        value: Some(event.specversion().as_str()),
                    })
                    .insert(Header {
                        key: "ce-type",
                        value: Some(event.type_()),
                    })
                    .insert(Header {
                        key: "ce-source",
                        value: Some(event.source()),
                    })
                    .insert(Header {
                        key: "ce-id",
                        value: Some(event.id()),
                    })
                    .insert(Header {
                        key: "ce-time",
                        value: event.time().map(|t| t.to_rfc3339()).as_ref(),
                    })
                    .insert(Header {
                        key: "ce-dataschema",
                        value: event.dataschema().map(|url| url.as_str().to_string()).as_deref(),
                    })
                    .insert(Header {
                        key: "ce-datacontenttype",
                        value: event.datacontenttype(),
                    }),
            )
            .payload(&payload);

        self.producer
            .send(record, self.timeout)
            .await
            .map_err(|(e, _)| Error::Publisher(e.to_string()))?;

        Ok(())
    }
}


#[async_trait]
impl Publisher for KafkaPublisher {
    async fn publish_event(&self, event: Event) -> Result<()> {
        self.publish(event).await
    }
}
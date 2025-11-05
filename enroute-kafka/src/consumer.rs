use std::{sync::Arc, pin::Pin, collections::HashMap};
use async_trait::async_trait;
use async_stream::stream;
use futures::{Stream, StreamExt};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rdkafka::{consumer::StreamConsumer, message::{Message, Headers, Header, BorrowedMessage}};

use enroute_core::{
    consumer::Consumer,
    event::Event,
    envelope::Envelope,
    error::{Error, Result},
};


fn try_get_header_str(msg: &BorrowedMessage, key: &str) -> Option<String> {
    msg.headers()?
        .iter()
        .find(|h| h.key == key)
        .and_then(|h| h.value)
        .map(|v| String::from_utf8_lossy(v).to_string())
}

fn get_header_str(msg: &BorrowedMessage, key: &str) -> Result<String> {
    try_get_header_str(msg, key)
        .ok_or_else(|| Error::Deserialization(format!("Missing {} header", key)))
}

// fn filtered_headers(msg: &BorrowedMessage, exclude_keys: &[&str]) -> HashMap<String, String> {
//     msg.headers()
//         .map(|headers| headers.iter()
//             .filter(|h| !exclude_keys.contains(&h.key))
//             .filter_map(|h| Some((h.key.to_string(), String::from_utf8_lossy(h.value?).to_string())))
//             .collect())
//         .unwrap_or_default()
// }

fn filtered_headers<F>(msg: &BorrowedMessage, filter: F) -> HashMap<String, String>
where
    F: FnMut(&Header<'_, &[u8]>) -> bool,
{
    msg.headers()
        .map(|headers| headers.iter()
            .filter(filter)
            .filter_map(|h| Some((h.key.to_string(), String::from_utf8_lossy(h.value?).to_string())))
            .collect())
        .unwrap_or_default()
}

pub struct KafkaConsumer {
    stream: Arc<StreamConsumer>,
}

impl KafkaConsumer {
    pub fn new(stream: StreamConsumer) -> Self {
        Self { stream: Arc::new(stream) }
    }
}

#[async_trait]
impl Consumer for KafkaConsumer {
    async fn stream_events(&self) -> Result<Pin<Box<dyn Stream<Item = Result<Envelope>> + Send>>> {
        let consumer = self.stream.clone();
        let stream = stream! {
            let mut message_stream = consumer.stream();

            while let Some(message) = message_stream.next().await {
                match message {
                    Ok(borrowed_msg) => yield Ok(Envelope::noop(
                        Event::builder()
                            .id(
                                borrowed_msg
                                    .key()
                                    .map(|k| String::from_utf8_lossy(k).to_string())
                                    .unwrap_or_else(|| Uuid::new_v4().to_string())
                            )
                            .time(
                                borrowed_msg
                                    .timestamp()
                                    .to_millis()
                                    .map(|ms| DateTime::<Utc>::from_timestamp_millis(ms))
                                    .flatten()
                                    .unwrap_or_else(|| Utc::now())
                            )
                            .type_(get_header_str(&borrowed_msg, "ce-type")?.as_str())
                            .source(get_header_str(&borrowed_msg, "ce-source")?.as_str()) 
                            .maybe_schema_url(
                                try_get_header_str(&borrowed_msg, "ce-dataschema")
                                    .as_deref()
                            )
                            .extensions(filtered_headers(
                                &borrowed_msg,
                                |h| ![
                                    "ce-type",
                                    "ce-source",
                                    "ce-id",
                                    "ce-time",
                                    "ce-specversion",
                                    "ce-dataschema",
                                    "ce-datacontenttype",
                                ].contains(&h.key)
                            ))
                            .build_raw(
                                borrowed_msg
                                    .payload()
                                    .unwrap_or_default()
                                    .to_vec()
                            )?,
                    )),
                    Err(e) => yield Err(Error::Consumer(e.to_string())),
                }
            }
        };

        Ok(Box::pin(stream))
    }
}

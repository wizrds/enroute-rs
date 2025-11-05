use std::{
    collections::HashMap,
    sync::Arc,
    fmt::Debug,
};
use futures::{SinkExt, channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded}};
use mea::rwlock::RwLock;

use enroute_core::{
    event::Event,
    error::{Error, Result},
};


#[derive(Debug)]
pub(crate) struct ConsumerGroup {
    consumers: Vec<UnboundedSender<Event>>,
    idx: usize,
}

impl ConsumerGroup {
    fn new() -> Self {
        Self {
            consumers: Vec::new(),
            idx: 0,
        }
    }

    fn add_consumer(&mut self) -> UnboundedReceiver<Event> {
        let (tx, rx) = unbounded();
        self.consumers.push(tx);
        rx
    }

    async fn dispatch(&mut self, event: &Event) -> Result<()> {
        if self.consumers.is_empty() {
            return Ok(());
        }

        let idx = self.idx % self.consumers.len();
        self.idx = (self.idx + 1) % self.consumers.len();

        self.consumers[idx]
            .send(event.clone())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}


#[derive(Debug)]
pub(crate) struct BrokerInner {
    groups: RwLock<HashMap<String, HashMap<String, Arc<RwLock<ConsumerGroup>>>>>,
}

impl BrokerInner {
    pub(crate) fn new() -> Self {
        Self {
            groups: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register_consumer(&self, channel: &str, consumer_tag: &str) -> UnboundedReceiver<Event> {
        self.groups
            .write()
            .await
            .entry(channel.to_string())
            .or_default()
            .entry(consumer_tag.to_string())
            .or_insert_with(|| Arc::new(RwLock::new(ConsumerGroup::new())))
            .write()
            .await
            .add_consumer()
    }

    pub async fn publish(&self, channel: &str, event: &Event) -> Result<()> {
        if let Some(consumer_tags) = self.groups.read().await.get(channel) {
            for group in consumer_tags.values() {
                group.write().await.dispatch(event).await?;
            }
        }

        Ok(())
    }
}

use std::{sync::Arc, fmt::Debug};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use enroute_core::{
    broker::{Broker, BrokerBuilder},
    consumer::ConsumerOptions,
    publisher::PublisherOptions,
    error::Result,
};

use crate::{
    inner::BrokerInner,
    publisher::InMemoryPublisher,
    consumer::InMemoryConsumer,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InMemoryBrokerConfig {
    requeue_on_nack: bool,
}

impl Default for InMemoryBrokerConfig {
    fn default() -> Self {
        Self {
            requeue_on_nack: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InMemoryBroker {
    config: InMemoryBrokerConfig,
    inner: Arc<BrokerInner>,
}

impl InMemoryBroker {
    pub fn new(config: InMemoryBrokerConfig) -> Self {
        Self {
            config,
            inner: Arc::new(BrokerInner::new()),
        }
    }

    pub fn builder() -> InMemoryBrokerBuilder {
        InMemoryBrokerBuilder::new()
    }
}

#[async_trait]
impl Broker for InMemoryBroker {
    type Publisher = InMemoryPublisher;
    type Consumer = InMemoryConsumer;

    async fn publisher(&self, options: PublisherOptions) -> Result<Self::Publisher> {
        Ok(InMemoryPublisher {
            channel: options.channel.to_string(),
            inner: self.inner.clone(),
        })
    }

    async fn consumer(&self, options: ConsumerOptions) -> Result<Self::Consumer> {
        Ok(InMemoryConsumer {
            channel: options.channel.to_string(),
            tag: options.consumer_tag.to_string(),
            requeue: self.config.requeue_on_nack,
            inner: self.inner.clone(),
        })
    }
}


pub struct InMemoryBrokerBuilder {
    requeue_on_nack: bool,
}

impl InMemoryBrokerBuilder {
    pub fn new() -> Self {
        Self {
            requeue_on_nack: false,
        }
    }

    pub fn with_requeue_on_nack(mut self, requeue: bool) -> Self {
        self.requeue_on_nack = requeue;
        self
    }
}

#[async_trait]
impl BrokerBuilder for InMemoryBrokerBuilder {
    type Broker = InMemoryBroker;

    async fn build(&self) -> Result<Self::Broker> {
        Ok(InMemoryBroker::new(
            InMemoryBrokerConfig {
                requeue_on_nack: self.requeue_on_nack,
            }
        ))
    }
}
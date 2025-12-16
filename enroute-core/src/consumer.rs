use std::{sync::Arc, pin::Pin};
use async_trait::async_trait;
use futures::Stream;
use serde::{Serialize, Deserialize};

use crate::{error::Result, envelope::Envelope};


/// Options for configuring a consumer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerOptions {
    /// The channel to consume messages from.
    pub channel: String,
    /// The consumer tag to identify the consumer.
    pub consumer_tag: String,
}

impl ConsumerOptions {
    /// Create a new [`ConsumerOptionsBuilder`] with default values.
    pub fn builder() -> ConsumerOptionsBuilder {
        ConsumerOptionsBuilder::default()
    }
}

/// A builder for creating [`ConsumerOptions`].
#[derive(Default, Debug, Clone)]
pub struct ConsumerOptionsBuilder {
    channel: Option<String>,
    consumer_tag: Option<String>,
}

impl ConsumerOptionsBuilder {
    /// Create a new [`ConsumerOptionsBuilder`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the channel to consume messages from.
    /// 
    /// # Arguments
    /// * `channel` - The name of the channel to consume messages from.
    /// 
    /// # Returns
    /// The builder with the channel set.
    pub fn channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    /// Set the consumer tag to identify the group this consumer
    /// is part of.
    /// 
    /// # Arguments
    /// * `consumer_tag` - The consumer tag to identify the consumer's group.
    /// 
    /// # Returns
    /// The builder with the consumer tag set.
    pub fn consumer_tag(mut self, consumer_tag: impl Into<String>) -> Self {
        self.consumer_tag = Some(consumer_tag.into());
        self
    }

    /// Build the [`ConsumerOptions`] from the builder.
    /// 
    /// # Returns
    /// The build [`ConsumerOptions`].
    /// 
    /// # Panics
    /// If the channel or consumer tag is not set.
    pub fn build(self) -> ConsumerOptions {
        ConsumerOptions {
            channel: self.channel.expect("channel is required"),
            consumer_tag: self.consumer_tag.expect("consumer_tag is required"),
        }
    }
}


/// A message consumer that can stream incoming messages.
#[async_trait]
pub trait Consumer: Send + Sync {
    /// Stream incoming message envelopes.
    /// 
    /// # Returns
    /// A result containing a stream of message envelopes or an error.
    async fn stream_events(&self) -> Result<Pin<Box<dyn Stream<Item = Result<Envelope>> + Send>>>;
}

/// A type-erased consumer that can hold any concrete consumer implementation.
pub struct AnyConsumer(Arc<dyn Consumer>);

impl AnyConsumer {
    pub fn new<C>(consumer: C) -> Self
    where
        C: Consumer + 'static,
    {
        Self(Arc::new(consumer))
    }

    pub fn into_inner(self) -> Arc<dyn Consumer> {
        self.0
    }
}

#[async_trait]
impl Consumer for AnyConsumer {
    async fn stream_events(&self) -> Result<Pin<Box<dyn Stream<Item = Result<Envelope>> + Send>>> {
        self.0.stream_events().await
    }
}

/// A trait for converting a concrete consumer into a type-erased [`AnyConsumer`].
pub trait IntoAnyConsumer: Consumer + 'static {
    fn into_any(self) -> AnyConsumer;
}

impl<C> IntoAnyConsumer for C
where
    C: Consumer + 'static,
{
    fn into_any(self) -> AnyConsumer {
        AnyConsumer::new(self)
    }
}

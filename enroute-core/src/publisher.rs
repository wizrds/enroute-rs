use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{error::Result, event::Event};


/// Options for configuring a publisher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherOptions {
    /// The channel to publish messages to.
    pub channel: String,
}

impl PublisherOptions {
    /// Create a new [`PublisherOptionsBuilder`] with default values.
    pub fn builder() -> PublisherOptionsBuilder {
        PublisherOptionsBuilder::new()
    }
}

/// A builder for creating [`PublisherOptions`].
#[derive(Default, Debug, Clone)]
pub struct PublisherOptionsBuilder {
    channel: Option<String>,
}

impl PublisherOptionsBuilder {
    /// Create a new [`PublisherOptionsBuilder`] with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the channel to publish messages to.
    /// 
    /// # Arguments
    /// * `channel` - The channel to publish messages to.
    /// 
    /// # Returns
    /// The builder with the channel set.
    pub fn channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    /// Build the [`PublisherOptions`] from the builder.
    /// 
    /// # Returns
    /// The built [`PublisherOptions`].
    /// 
    /// # Panics
    /// If the channel is not set.
    pub fn build(self) -> PublisherOptions {
        PublisherOptions {
            channel: self.channel.expect("channel is required"),
        }
    }
}

/// A message publisher that can publish events.
#[async_trait]
pub trait Publisher: Send + Sync {
    /// Publish an event to the message broker.
    /// 
    /// # Arguments
    /// * `event` - The event to be published.
    /// 
    /// # Returns
    /// A result indicating success or failure.
    async fn publish_event(&self, event: Event) -> Result<()>;
}


/// A type-erased publisher that can hold any concrete publisher implementation.
pub struct AnyPublisher(Arc<dyn Publisher>);

impl AnyPublisher {
    pub fn new<P>(publisher: P) -> Self
    where
        P: Publisher + 'static,
    {
        Self(Arc::new(publisher))
    }

    pub fn into_inner(self) -> Arc<dyn Publisher> {
        self.0
    }
}

#[async_trait]
impl Publisher for AnyPublisher {
    async fn publish_event(&self, event: Event) -> Result<()> {
        self.0.publish_event(event).await
    }
}

/// A trait for converting a concrete publisher into a type-erased [`AnyPublisher`].
pub trait IntoAnyPublisher: Publisher + 'static {
    fn into_any(self) -> AnyPublisher;
}

impl<P> IntoAnyPublisher for P
where
    P: Publisher + 'static,
{
    fn into_any(self) -> AnyPublisher {
        AnyPublisher::new(self)
    }
}

use std::{sync::Arc, pin::Pin};
use async_trait::async_trait;
use futures::Stream;
use serde::{Serialize, Deserialize};

use crate::{error::Result, envelope::Envelope};


/// Options for configuring a consumer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerOptions {
    /// The channel to consume messages from.
    pub channel: &'static str,
    /// The consumer tag to identify the consumer.
    pub consumer_tag: &'static str,
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

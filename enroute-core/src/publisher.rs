use std::sync::Arc;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{error::Result, event::Event};


/// Options for configuring a publisher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherOptions {
    /// The channel to publish messages to.
    pub channel: &'static str,
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

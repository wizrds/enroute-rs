use std::sync::Arc;
use async_trait::async_trait;

use crate::{
    error::Result,
    publisher::{Publisher, AnyPublisher, IntoAnyPublisher, PublisherOptions},
    consumer::{Consumer, AnyConsumer, IntoAnyConsumer, ConsumerOptions},
};


/// A message broker that can create publishers and consumers.
#[async_trait]
pub trait Broker: Send + Sync {
    type Publisher: Publisher;
    type Consumer: Consumer;

    /// Create a publisher with the given options.
    /// 
    /// # Arguments
    /// * `options` - The options to configure the publisher.
    /// 
    /// # Returns
    /// A result containing the created publisher or an error.
    async fn publisher(&self, options: PublisherOptions) -> Result<Self::Publisher>;
    /// Create a consumer with the given options.
    /// 
    /// # Arguments
    /// * `options` - The options to configure the consumer.
    /// 
    /// # Returns
    /// A result containing the created consumer or an error.
    async fn consumer(&self, options: ConsumerOptions) -> Result<Self::Consumer>;
    /// Create a publisher and consumer pair with the given options.
    ///
    /// # Arguments
    /// * `options` - A tuple containing the options to configure the publisher and consumer.
    /// 
    /// # Returns
    /// A result containing the created publisher and consumer pair or an error.
    async fn pair(&self, options: (PublisherOptions, ConsumerOptions)) -> Result<(Self::Publisher, Self::Consumer)> {
        Ok((
            self.publisher(options.0).await?,
            self.consumer(options.1).await?,
        ))
    }
}

/// A builder for creating brokers.
#[async_trait]
pub trait BrokerBuilder: Send + Sync {
    type Broker: Broker;

    async fn build(&self) -> Result<Self::Broker>;
}

/// A type-erased broker that can hold any concrete broker implementation.
pub struct AnyBroker(Arc<dyn Broker<Publisher = AnyPublisher, Consumer = AnyConsumer>>);

impl AnyBroker {
    pub fn new<B>(broker: B) -> Self
    where
        B: Broker + 'static,
        B::Publisher: Publisher + IntoAnyPublisher,
        B::Consumer: Consumer + IntoAnyConsumer,
    {
        Self(Arc::new(BrokerAdapter { inner: broker }))
    }

    pub fn into_inner(self) -> Arc<dyn Broker<Publisher = AnyPublisher, Consumer = AnyConsumer>> {
        self.0
    }
}

#[async_trait]
impl Broker for AnyBroker {
    type Publisher = AnyPublisher;
    type Consumer = AnyConsumer;

    async fn publisher(&self, options: PublisherOptions) -> Result<Self::Publisher> {
        self.0.publisher(options).await
    }

    async fn consumer(&self, options: ConsumerOptions) -> Result<Self::Consumer> {
        self.0.consumer(options).await
    }
}

struct BrokerAdapter<B: Broker> {
    inner: B,
}

#[async_trait]
impl<B> Broker for BrokerAdapter<B>
where
    B: Broker + Send + Sync,
    B::Publisher: Publisher + IntoAnyPublisher,
    B::Consumer: Consumer + IntoAnyConsumer,
{
    type Publisher = AnyPublisher;
    type Consumer = AnyConsumer;

    async fn publisher(&self, options: PublisherOptions) -> Result<Self::Publisher> {
        Ok(self.inner.publisher(options).await?.into_any())
    }

    async fn consumer(&self, options: ConsumerOptions) -> Result<Self::Consumer> {
        Ok(self.inner.consumer(options).await?.into_any())
    }
}

/// A trait for converting a broker into a type-erased [`AnyBroker`].
pub trait IntoAnyBroker {
    fn into_any(self) -> AnyBroker;
}

impl<B> IntoAnyBroker for B
where
    B: Broker + 'static,
    B::Publisher: Publisher + IntoAnyPublisher,
    B::Consumer: Consumer + IntoAnyConsumer,
{
    fn into_any(self) -> AnyBroker {
        AnyBroker::new(self)
    }
}

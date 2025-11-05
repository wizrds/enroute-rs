pub use enroute_core::{
    error::{Error, Result},
    event::{EventData, EventBuilder, Event},
    envelope::{Envelope, Acker},
    broker::{Broker, AnyBroker, IntoAnyBroker, BrokerBuilder},
    publisher::{Publisher, AnyPublisher, IntoAnyPublisher, PublisherOptions},
    consumer::{Consumer, AnyConsumer, IntoAnyConsumer, ConsumerOptions},
};
pub use enroute_macros::EventData;

pub mod memory {
    pub use enroute_memory::{
        broker::{InMemoryBroker, InMemoryBrokerBuilder, InMemoryBrokerConfig},
        publisher::InMemoryPublisher,
        consumer::InMemoryConsumer,
        acker::InMemoryAcker,
    };
}

#[cfg(feature = "kafka")]
pub mod kafka {
    pub use enroute_kafka::{
        broker::{KafkaBroker, KafkaBrokerBuilder, KafkaBrokerConfig},
        publisher::KafkaPublisher,
        consumer::KafkaConsumer,
    };
}
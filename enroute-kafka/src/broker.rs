use std::{time::Duration, collections::HashMap};
use anyhow::anyhow;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use rdkafka::{ClientConfig, producer::FutureProducer, consumer::{Consumer, StreamConsumer}};

use enroute_core::{
    publisher::PublisherOptions,
    consumer::ConsumerOptions,
    broker::{Broker, BrokerBuilder},
    error::{Error, Result},
};

use enroute_kafka::{
    consumer::KafkaConsumer,
    publisher::KafkaPublisher,
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaBrokerConfig {
    pub bootstrap_servers: Vec<String>,
    pub producer_timeout_ms: Option<Duration>,
}

impl KafkaBrokerConfig {
    pub fn into_client_config(&self, options: Option<HashMap<String, String>>) -> ClientConfig {
        let mut config = ClientConfig::new();

        config.set("bootstrap.servers", &self.bootstrap_servers.join(","));

        if let Some(timeout) = self.producer_timeout_ms {
            config.set("message.timeout.ms", &timeout.as_millis().to_string());
        }

        config.extend(options.unwrap_or_default().into_iter());

        config
    }
}


#[derive(Clone)]
pub struct KafkaBroker {
    config: KafkaBrokerConfig,
}

impl KafkaBroker {
    pub fn new(config: KafkaBrokerConfig) -> Self {
        Self { config }
    }

    pub fn builder() -> KafkaBrokerBuilder {
        KafkaBrokerBuilder::new()
    }

    pub fn new_producer(&self) -> Result<FutureProducer> {
        Ok(
            self.config
                .into_client_config(None)
                .create::<FutureProducer>()
                .map_err(|e| Error::Unknown(anyhow!(e)))?
        )
    }

    pub fn new_consumer(&self, topic: &str, consumer_tag: &str) -> Result<StreamConsumer> {
        let consumer = self.config
            .into_client_config(Some(HashMap::from([
                ("group.id".to_string(), consumer_tag.to_string()),
                ("enable.auto.commit".to_string(), "true".to_string()),
                ("auto.offset.reset".to_string(), "earliest".to_string()),
            ])))
            .create::<StreamConsumer>()
            .map_err(|e| Error::Unknown(anyhow!(e)))?;

        consumer
            .subscribe(&[topic])
            .map_err(|e| Error::Unknown(anyhow!(e)))?;

        Ok(consumer)
    }
}

#[async_trait]
impl Broker for KafkaBroker {
    type Publisher = KafkaPublisher;
    type Consumer = KafkaConsumer;

    async fn publisher(&self, options: PublisherOptions) -> Result<Self::Publisher> {
        Ok(KafkaPublisher::new(
            self.new_producer()?,
            options.channel.to_string(),
            self.config.producer_timeout_ms,
        ))
    }

    async fn consumer(&self, options: ConsumerOptions) -> Result<Self::Consumer> {
        Ok(KafkaConsumer::new(
            self.new_consumer(options.channel, options.consumer_tag)?
        ))
    }
}


pub struct KafkaBrokerBuilder {
    bootstrap_servers: Option<Vec<String>>,
    producer_timeout_ms: Option<Duration>,
}

impl KafkaBrokerBuilder {
    pub fn new() -> Self {
        Self {
            bootstrap_servers: None,
            producer_timeout_ms: None,
        }
    }

    pub fn with_bootstrap_servers(mut self, servers: Vec<String>) -> Self {
        self.bootstrap_servers = Some(servers);
        self
    }

    pub fn with_producer_timeout_ms(mut self, timeout: Duration) -> Self {
        self.producer_timeout_ms = Some(timeout);
        self
    }
}

#[async_trait]
impl BrokerBuilder for KafkaBrokerBuilder {
    type Broker = KafkaBroker;

    async fn build(&self) -> Result<Self::Broker> {
        Ok(KafkaBroker::new(KafkaBrokerConfig {
            bootstrap_servers: self.bootstrap_servers
                .clone()
                .ok_or_else(|| Error::Builder("missing bootstrap_servers".to_string()))?,
            producer_timeout_ms: self.producer_timeout_ms,
        }))
    } 
}
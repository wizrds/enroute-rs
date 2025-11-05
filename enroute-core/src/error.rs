use thiserror::{Error as ThisError};


#[derive(ThisError, Debug)]
pub enum Error {
    /// An error occurred during serialization of the event.
    #[error("Serialization error: {0}")]
    Serialization(String),
    /// An error occurred during deserialization of the event.
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    /// Missing event data in the envelope.
    #[error("Missing event data")]
    MissingEventData,
    /// An error occurred in the publisher.
    #[error("Publisher error: {0}")]
    Publisher(String),
    /// An error occurred in the consumer.
    #[error("Consumer error: {0}")]
    Consumer(String),
    /// An error occurred in the broker builder.
    #[error("Builder error: {0}")]
    Builder(String),
    /// An unknown error occurred.
    #[error("Unknown error: {0}")]
    Unknown(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

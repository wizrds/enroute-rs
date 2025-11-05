use std::{sync::Arc, fmt::Debug};
use async_trait::async_trait;

use crate::event::Event;


/// An acker that can acknowledge or negatively acknowledge message processing.
#[async_trait]
pub trait Acker: Send + Sync + Debug {
    /// Acknowledge successful message processing.
    async fn ack(&self);
    /// Negatively acknowledge failed message processing.
    async fn nack(&self);
}

/// A no-operation acker that does nothing on ack or nack.
#[derive(Debug, Clone)]
pub struct NoOpAcker;

#[async_trait]
impl Acker for NoOpAcker {
    async fn ack(&self) {}
    async fn nack(&self) {}
}


/// An envelope that wraps an event and its associated acker.
#[derive(Debug, Clone)]
pub struct Envelope {
    event: Event,
    acker: Arc<dyn Acker>,
}

impl Envelope {
    pub fn new(event: Event, acker: Arc<dyn Acker>) -> Self {
        Self { event, acker }
    }

    /// Create a noop envelope with a no-operation acker.
    /// 
    /// # Arguments
    /// * `event` - The event to be wrapped in the envelope.
    /// 
    /// # Returns
    /// A noop envelope containing the event.
    pub fn noop(event: Event) -> Self {
        Self { event, acker: Arc::new(NoOpAcker) }
    }

    /// Get a reference to the event contained in the envelope.
    /// 
    /// # Returns
    /// A reference to the event.
    pub fn event(&self) -> &Event {
        &self.event
    }

    /// Acknowledge successful processing of the event.
    pub async fn ack(&self) {
        self.acker.ack().await;
    }

    /// Negatively acknowledge failed processing of the event.
    pub async fn nack(&self) {
        self.acker.nack().await;
    }
}
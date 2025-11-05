use std::{
    sync::{Arc, Weak, atomic::{AtomicBool, Ordering}},
    fmt::Debug,
};
use async_trait::async_trait;

use enroute_core::{event::Event, envelope::Acker};

use crate::inner::BrokerInner;


#[derive(Debug, Clone)]
pub struct InMemoryAcker {
    broker_inner: Weak<BrokerInner>,
    channel: String,
    event: Event,
    done: Arc<AtomicBool>,
    requeue: bool,
}

impl InMemoryAcker {
    pub(crate) fn new(
        broker_inner: Weak<BrokerInner>,
        channel: String,
        event: Event,
        requeue: bool,
    ) -> Self {
        Self {
            broker_inner,
            channel,
            event,
            done: Arc::new(AtomicBool::new(false)),
            requeue,
        }
    }
}

#[async_trait]
impl Acker for InMemoryAcker {
    async fn ack(&self) {
        if self.done.swap(true, Ordering::SeqCst) {
            return;
        }
    }

    async fn nack(&self) {
        if self.done.swap(true, Ordering::SeqCst) {
            return;
        }

        if self.requeue {
            if let Some(inner) = self.broker_inner.upgrade() {
                let _ = inner
                    .publish(&self.channel, &self.event)
                    .await;
            }
        }
    }
}

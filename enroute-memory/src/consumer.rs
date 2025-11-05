use std::{sync::Arc, pin::Pin};
use async_trait::async_trait;
use futures::{Stream, StreamExt};

use enroute_core::{
    consumer::Consumer,
    envelope::Envelope,
    error::Result,
};

use crate::{inner::BrokerInner, acker::InMemoryAcker};


#[derive(Clone)]
pub struct InMemoryConsumer {
    pub(crate) channel: String,
    pub(crate) tag: String,
    pub(crate) requeue: bool,
    pub(crate) inner: Arc<BrokerInner>,
}

#[async_trait]
impl Consumer for InMemoryConsumer {
    async fn stream_events(&self) -> Result<Pin<Box<dyn Stream<Item = Result<Envelope>> + Send>>> {
        let inner_weak = Arc::downgrade(&self.inner);
        let channel_name = self.channel.clone();
        let requeue = self.requeue.clone();

        Ok(Box::pin(
            self.inner
                .register_consumer(&self.channel, &self.tag)
                .await
                .filter_map(move |event| {
                    let inner_weak = inner_weak.clone();
                    let channel_name = channel_name.clone();

                    async move {
                        Some(Ok(Envelope::new(
                            event.clone(),
                            Arc::new(InMemoryAcker::new(
                                inner_weak,
                                channel_name,
                                event,
                                requeue,
                            ))
                        )))
                    }
                })
        ))
    }
}
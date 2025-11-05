use std::sync::Arc;
use async_trait::async_trait;

use enroute_core::{event::Event, error::Result, publisher::Publisher};

use crate::inner::BrokerInner;


#[derive(Clone)]
pub struct InMemoryPublisher {
    pub(crate) channel: String,
    pub(crate) inner: Arc<BrokerInner>,
}

#[async_trait]
impl Publisher for InMemoryPublisher {
    async fn publish_event(&self, event: Event) -> Result<()> {
        self.inner
            .publish(&self.channel, &event)
            .await?;

        Ok(())
    }
}

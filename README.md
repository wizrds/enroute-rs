# Enroute


## Overview
**Enroute** is a modular, event-driven framework for Rust that provides a unified abstraction for publishing, consuming, and acknowledging messages across different backends such as **in-memory** and **Kafka**.

It is built around the [CloudEvents](https://cloudevents.io) specification to ensure consistency, strong typing, and interoperability across distributed systems.  
The framework’s design emphasizes **composability**, **testability**, and **simplicity**, allowing developers to define, emit, and process events without being tightly coupled to any specific message broker.

## Features
- 🧩 **Unified abstractions** — consistent API for all brokers via `Broker`, `Publisher`, and `Consumer` traits.  
- ⚡ **Multiple backends** — built-in support for in-memory and Kafka brokers with more to come.  
- 🧠 **Strong typing** — define event payloads with the `#[derive(EventData)]` macro for compile-time safety.  
- 🧱 **Modular design** — each crate focuses on a single responsibility (`core`, `memory`, `kafka`, `macros`).  
- 🧪 **Great for testing** — the in-memory broker makes it easy to simulate and validate event flows in tests.  

## Installation

```bash
cargo add enroute --git https://github.com/wizrds/enroute-rs.git
```

## Usage

### Basic Example

```rust
use serde::{Serialize, Deserialize};
use futures::StreamExt;
use enroute::{memory::InMemoryBroker, Event, EventData, PublisherOptions, ConsumerOptions};

#[derive(Clone, Debug, Serialize, Deserialize, EventData)]
#[event_data(event_type = "user.created", channel_name = "public.myapp.user.created")]
struct UserCreated {
    pub id: u64,
    pub name: String,
}

#[tokio::main]
async fn main() {
    let broker = InMemoryBroker::builder()
        .build()
        .await
        .expect("Failed to create broker");

    let user_created_publisher = broker
        .publisher(
            PublisherOptions::builder()
                .channel(UserCreated::channel_name())
                .build(),
        )
        .await
        .expect("Failed to create publisher");

    let user_created_consumer = broker
        .consumer(
            ConsumerOptions::builder()
                .channel(UserCreated::channel_name())
                .consumer_tag("user_created_consumer")
                .build(),
        )
        .await
        .expect("Failed to create consumer");

    let mut event_stream = user_created_consumer
        .stream_events()
        .await
        .expect("Failed to create event stream");

    user_created_publisher
        .publish_event(
            Event::builder()
                .source("myapp")
                .id("event-123")
                .schema_url("http://example.com/schemas/user_created.json")
                .extension("com.example.custom", "value")
                .build(UserCreated {
                    id: 1,
                    name: "Alice".to_string(),
                })
                .expect("Failed to build event")
        )
        .await
        .expect("Failed to publish event");

    let envelope = event_stream
        .next()
        .await
        .expect("Failed to receive event");

    if envelope.event().type_() == UserCreated::event_type() {
        println!("Received UserCreated event: {:?}", envelope.event().data::<UserCreated>());
    }

    // Note: Acknowledgement (or nack) depends on the broker implementation,
    // often just a noop
    envelope.ack().await.expect("Failed to acknowledge event");
}
```

### Kafka Example

To use the Kafka backend, ensure you set the feature flag in your `Cargo.toml`:

```toml
[dependencies]
enroute = { git = "https://github.com/wizrds/enroute-rs.git", features = ["kafka"] }
```

```rust
use serde::{Serialize, Deserialize};
use futures::StreamExt;
use enroute::{kafka::KafkaBroker, Event, EventData, PublisherOptions, ConsumerOptions};

#[derive(Clone, Debug, Serialize, Deserialize, EventData)]
#[event_data(event_type = "order.placed", channel_name = "public.myapp.order.placed")]
struct OrderPlaced {
    pub order_id: u64,
    pub amount: f64,
}

#[tokio::main]
async fn main() {
    let broker = KafkaBroker::builder()
        .with_bootstrap_servers(vec!["localhost:9092"])
        .build()
        .await
        .expect("Failed to create Kafka broker");

    let order_placed_publisher = broker
        .publisher(
            PublisherOptions::builder()
                .channel(OrderPlaced::channel_name())
                .build(),
        )
        .await
        .expect("Failed to create publisher");

    let order_placed_consumer = broker
        .consumer(
            ConsumerOptions::builder()
                .channel(OrderPlaced::channel_name())
                .consumer_tag("order_placed_consumer")
                .build(),
        )
        .await
        .expect("Failed to create consumer");

    let mut event_stream = order_placed_consumer
        .stream_events()
        .await
        .expect("Failed to create event stream");

    order_placed_publisher
        .publish_event(
            Event::builder()
                .source("myapp")
                .id("event-456")
                .schema_url("http://example.com/schemas/order_placed.json")
                .extension("com.example.custom", "value")
                .build(OrderPlaced {
                    order_id: 1001,
                    amount: 250.75,
                })
                .expect("Failed to build event")
        )
        .await
        .expect("Failed to publish event");

    let envelope = event_stream
        .next()
        .await
        .expect("Failed to receive event");

    if envelope.event().type_() == OrderPlaced::event_type() {
        println!("Received OrderPlaced event: {:?}", envelope.event().data::<OrderPlaced>());
    }

    envelope.ack().await.expect("Failed to acknowledge event");
}
```

## License
This project is licensed under ISC License.

## Support & Feedback
If you encounter any issues or have feedback, please open an issue.

Made with ❤️ by Tim Pogue

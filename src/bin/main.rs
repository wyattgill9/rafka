use std::sync::Arc;

use anyhow::Result;
use broker::{HookRegistry, SimpleBroker};
use consumer::simple_consumer::SimpleConsumer;
use my_core::Consumer;
use my_core::Message;
use my_core::Producer;
use producer::SimpleProducer;
use storage::InMemoryStorage;

#[derive(Debug, Clone)]
struct MyMessage {
    payload: Vec<u8>,
    timestamp: u64,
}

impl Message for MyMessage {
    fn payload(&self) -> &[u8] {
        &self.payload
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the storage engine
    let storage = Arc::new(InMemoryStorage::<MyMessage>::new());

    // Initialize the hook registry
    let hook_registry = Arc::new(HookRegistry::new());

    // Create the broker
    let broker = Arc::new(SimpleBroker::new(storage.clone(), hook_registry.clone()));

    // Create a producer
    let producer_id = "producer_1".to_string();
    let producer = SimpleProducer::new(broker.clone(), producer_id).await?;

    // Create a consumer
    let consumer_id = "consumer_1".to_string();
    let mut consumer: SimpleConsumer<MyMessage, SimpleBroker<InMemoryStorage<MyMessage>>> =
        SimpleConsumer::new(broker.clone(), consumer_id).await?;

    // Send messages via the producer
    for i in 0..10 {
        let message = MyMessage {
            payload: format!("Message {}", i).into_bytes(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };
        producer.send(message).await?;
    }

    // Flush the producer to ensure all messages are sent
    producer.flush().await?;

    // Spawn a task to consume messages
    let consumer_task = tokio::spawn(async move {
        while let Some(message) = consumer.poll().await.unwrap() {
            println!(
                "Received message: {}",
                String::from_utf8_lossy(message.payload())
            );
            // Commit the message (optional)
            consumer.commit().await.unwrap();
        }
    });

    // Wait for the consumer task to finish processing
    consumer_task.await?;

    Ok(())
}

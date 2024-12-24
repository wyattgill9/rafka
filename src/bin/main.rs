use rafka_broker::Broker;
use rafka_cli::{Commands, CLI};
use rafka_consumer::Consumer;
use rafka_producer::Producer;
use rafka_storage::db::RetentionPolicy;
use std::time::Duration;

type Resulty = Result<(), Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Resulty {
    let command = CLI::get_parse();

    match command {
        Commands::Consumer { port, partition } => start_consumer(port, partition).await,
        Commands::Broker {
            port,
            partition,
            total_partition,
            retention_secs,
        } => start_broker(port, partition, total_partition, retention_secs).await,
        Commands::Producer {
            brokers,
            key,
            message,
            topic,
        } => start_producer(brokers, message, key, topic).await,
    }
}

async fn start_broker(
    port: u16,
    partition: u32,
    total_partition: u32,
    retention_secs: u64,
) -> Resulty {
    let retention_policy = RetentionPolicy {
        max_age: Duration::from_secs(retention_secs),
        max_bytes: 1024 * 1024 * 1024, // 1GB default
    };

    println!(
        "Starting Rafka broker on 127.0.0.1:{} (partition {}/{})",
        port, partition, total_partition
    );

    let broker = Broker::new(partition, total_partition);
    broker.serve(&format!("127.0.0.1:{}", port)).await?;
    Ok(())
}

async fn start_consumer(port: u16, partition: u32) -> Resulty {
    let mut consumer = Consumer::new(&format!("127.0.0.1:{}", port)).await?;

    consumer.subscribe("greetings".to_string()).await?;

    println!(
        "Consumer ready - listening for messages on 'greetings' topic (partition {})",
        partition
    );

    let mut rx = consumer.consume("greetings".to_string()).await?;

    while let Some(message) = rx.recv().await {
        println!("Received message: {}", message);
    }

    Ok(())
}

async fn start_producer(
    brokers: Vec<String>,
    message: String,
    key: String,
    topic: String,
) -> Resulty {
    println!(
        "Publishing to 'greetings' topic with key '{}': {}",
        key, message
    );

    let mut producer = Producer::new(&brokers[0]).await?;

    producer
        .publish("greetings".to_string(), message, key)
        .await?;

    Ok(())
}

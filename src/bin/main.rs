use rafka_broker::Broker;
use rafka_cli::{Commands, CLI};
use rafka_consumer::Consumer;
use rafka_core::proto::rafka::{broker_service_client::BrokerServiceClient, GetMetricsRequest};
use rafka_producer::Producer;
use rafka_storage::db::RetentionPolicy;
use std::time::Duration;

type Resulty = Result<(), Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> Resulty {
    let command = CLI::get_parse();

    match command {
        Commands::Metrics { port } => check_metrics(port).await,
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
        } => start_producer(brokers, message, key).await,
    }
}

async fn check_metrics(port: u16) -> Resulty {
    let mut client = BrokerServiceClient::connect(format!("http://127.0.0.1:{}", port)).await?;

    // Get metrics
    let response = client.get_metrics(GetMetricsRequest {}).await?;

    let metrics = response.into_inner();
    println!("Storage Metrics:");
    println!("Total Messages: {}", metrics.total_messages);
    println!("Total Size: {} bytes", metrics.total_bytes);
    println!(
        "Oldest Message Age: {} seconds",
        metrics.oldest_message_age_secs
    );

    Ok(())
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

    let broker = Broker::new(partition, total_partition, Some(retention_policy));
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

async fn start_producer(brokers: Vec<String>, message: String, key: String) -> Resulty {
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

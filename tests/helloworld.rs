mod common;

#[cfg(test)]
mod module {

    use std::time::Duration;

    use rafka_consumer::Consumer;
    use rafka_producer::Producer;
    use tokio::{task, time::sleep};

    use crate::common::{setup_brokers, DEFAULT_ADDRESS};

    #[tokio::test]
    async fn test() {
        const TOPIC: &str = "greetings";
        const MESSAGE: &str = "Hello, world!";
        const KEY: &str = "default-key";

        // This threads doesnt end, so whe don't wait for it
        let _ = task::spawn(async { setup_brokers(1, 1).await });

        // Time for broker start
        sleep(Duration::from_millis(50)).await;

        let consumer_task = task::spawn(async {
            let topic = String::from(TOPIC);

            let mut consumer = Consumer::new(DEFAULT_ADDRESS).await.unwrap();
            consumer.subscribe(topic.clone()).await.unwrap();
            let mut rx = consumer.consume(topic).await.unwrap();

            while let Some(message) = rx.recv().await {
                assert_eq!(message, MESSAGE);
                return;
            }
        });

        // Time for consumers start
        sleep(Duration::from_millis(50)).await;

        let producer_task = task::spawn(async {
            let mut producer = Producer::new(DEFAULT_ADDRESS).await.unwrap();

            producer
                .publish(
                    String::from(TOPIC),
                    String::from(MESSAGE),
                    String::from(KEY),
                )
                .await
                .unwrap();
        });

        producer_task.await.unwrap();
        consumer_task.await.unwrap();
    }
}

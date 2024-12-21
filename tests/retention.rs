mod common;

#[cfg(test)]
mod module {

    use std::time::Duration;

    use crate::common::{setup_brokers, DEFAULT_ADDRESS};
    use rafka_consumer::Consumer;
    use tokio::{task, time::sleep};

    #[tokio::test]
    async fn test() {
        const RETENTION_SECS: usize = 10;
        const KEY: &str = "default-key";

        let _ = task::spawn(async { setup_brokers(1, RETENTION_SECS).await });

        let mut threads = vec![];

        // Time for broker start
        sleep(Duration::from_millis(50)).await;

        let consumer_task = task::spawn(async {
            let topic = String::from(KEY);

            let mut consumer = Consumer::new(DEFAULT_ADDRESS).await.unwrap();
            consumer.subscribe(topic.clone()).await.unwrap();
            let mut rx = consumer.consume(topic).await.unwrap();

            while let Some(message) = rx.recv().await {
                assert_eq!(message, "message");
                return;
            }
        });

        threads.push(consumer_task);
    }
}

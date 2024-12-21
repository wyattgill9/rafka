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
        const EXPECTED_MESSAGES: [&str; 6] = [
            "Batch1-Message0",
            "Batch1-Message1",
            "Batch1-Message2",
            "Batch2-Message3",
            "Batch2-Message4",
            "Batch2-Message5",
        ];
        const KEYS: [&str; 6] = ["key-0", "key-1", "key-2", "key-3", "key-4", "key-5"];

        let _ = task::spawn(async { setup_brokers(1, 1).await });

        sleep(Duration::from_millis(50)).await;

        let mut threads = vec![];

        let consumer_task = task::spawn(async {
            let topic = String::from(TOPIC);

            let mut consumer = Consumer::new(DEFAULT_ADDRESS).await.unwrap();
            consumer.subscribe(topic.clone()).await.unwrap();
            let mut rx = consumer.consume(topic).await.unwrap();

            let mut index = 0;
            while let Some(message) = rx.recv().await {
                assert_eq!(message, EXPECTED_MESSAGES[index]);

                index += 1;
                if index == 5 {
                    return;
                }
            }
        });

        threads.push(consumer_task);

        sleep(Duration::from_millis(50)).await;

        for i in 0..3 {
            let producer_task = task::spawn(async move {
                let mut producer = Producer::new(DEFAULT_ADDRESS).await.unwrap();
                producer
                    .publish(
                        String::from(TOPIC),
                        String::from(EXPECTED_MESSAGES[i]),
                        String::from(KEYS[i]),
                    )
                    .await
                    .unwrap();
            });

            threads.push(producer_task);
        }

        sleep(Duration::from_millis(50)).await;

        for i in 3..6 {
            let producer_task = task::spawn(async move {
                let mut producer = Producer::new(DEFAULT_ADDRESS).await.unwrap();
                producer
                    .publish(
                        String::from(TOPIC),
                        String::from(EXPECTED_MESSAGES[i]),
                        String::from(KEYS[i]),
                    )
                    .await
                    .unwrap();
            });

            threads.push(producer_task);
        }

        for thread in threads {
            thread.await.unwrap();
        }
    }
}

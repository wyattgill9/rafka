mod common;

#[cfg(test)]
mod module {
    use std::time::Duration;

    use rafka_consumer::Consumer;
    use rafka_producer::Producer;
    use tokio::{task, time::sleep};

    use crate::common::{setup_brokers, PORT};

    #[tokio::test]
    async fn test() {
        const BROKER_COUNT: usize = 3;
        const TOPIC: &str = "partitioned-topic";
        const MESSAGE_COUNT: usize = 9;
        const MESSAGES: [&str; 10] = [
            "Message-0",
            "Message-1",
            "Message-2",
            "Message-3",
            "Message-4",
            "Message-5",
            "Message-6",
            "Message-7",
            "Message-8",
            "Message-9",
        ];

        let _ = task::spawn(async { setup_brokers(BROKER_COUNT).await });

        sleep(Duration::from_millis(50)).await;

        let mut threads = vec![];

        for i in 0..BROKER_COUNT {
            let mut index = i;
            let topic = String::from(TOPIC);

            let consumer_thread = task::spawn(async move {
                let address = &format!("127.0.0.1:{}", PORT + i);

                let mut consumer = Consumer::new(address).await.unwrap();
                consumer.subscribe(topic.clone()).await.unwrap();
                let mut rx = consumer.consume(topic).await.unwrap();

                while let Some(message) = rx.recv().await {
                    assert_eq!(message, MESSAGES[index]);

                    index += BROKER_COUNT;

                    if index >= 9 {
                        return;
                    }
                }
            });

            threads.push(consumer_thread);
        }

        sleep(Duration::from_millis(50)).await;

        for i in 0..MESSAGE_COUNT {
            let producer_task = task::spawn(async move {
                let address = &format!("127.0.0.1:{}", (PORT + (i % BROKER_COUNT)));

                let mut producer = Producer::new(address).await.unwrap();

                producer
                    .publish(
                        String::from(TOPIC),
                        String::from(MESSAGES[i]),
                        String::from("default-key"),
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

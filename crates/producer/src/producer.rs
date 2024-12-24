use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum BrokerMessage {
    Publish {
        key: String,
        topic: String,
        payload: Vec<u8>,
    },
    Register {
        client_id: String,
        client_type: String,
    },
}

pub struct Producer {
    stream: TcpStream,
    producer_id: String,
}

impl Producer {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(addr).await?;
        let producer_id = Uuid::new_v4().to_string();

        let mut producer = Self {
            stream,
            producer_id,
        };

        // Register with broker
        let register_msg = BrokerMessage::Register {
            client_id: producer.producer_id.clone(),
            client_type: "producer".to_string(),
        };

        producer.send_message(&register_msg).await?;
        let response = producer.read_response().await?;

        println!("Producer registered with ID: {}", producer.producer_id);
        println!("Registration response: {}", response);

        Ok(producer)
    }

    async fn send_message(&mut self, message: &BrokerMessage) -> Result<(), Box<dyn Error>> {
        let message_bytes = serde_json::to_vec(message)?;
        self.stream.write_all(&message_bytes).await?;
        Ok(())
    }

    async fn read_response(&mut self) -> Result<String, Box<dyn Error>> {
        let mut buffer = vec![0; 1024];
        let n = self.stream.read(&mut buffer).await?;
        let response = String::from_utf8(buffer[..n].to_vec())?;
        Ok(response)
    }

    pub async fn publish(
        &mut self,
        topic: String,
        message: String,
        key: String,
    ) -> Result<(), Box<dyn Error>> {
        let publish_msg = BrokerMessage::Publish {
            key,
            topic,
            payload: message.into_bytes(),
        };

        self.send_message(&publish_msg).await?;
        let response = self.read_response().await?;

        println!("Response from broker: {}", response);
        Ok(())
    }

    // util method to publish batch of messages
    pub async fn publish_batch(
        &mut self,
        topic: String,
        messages: Vec<(String, String)>, // (key, message) pairs
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut responses = Vec::new();

        for (key, message) in messages {
            let publish_msg = BrokerMessage::Publish {
                key,
                topic: topic.clone(),
                payload: message.into_bytes(),
            };

            self.send_message(&publish_msg).await?;
            let response = self.read_response().await?;
            responses.push(response);
        }

        Ok(responses)
    }

    // Get a new stream for parallel publishing if needed
    pub async fn clone_connection(&mut self) -> Result<Self, Box<dyn Error>> {
        let std_socket = self.stream.into_std()?;
        self.stream = TcpStream::from_std(std_socket.try_clone()?)?;
        let stream = TcpStream::from_std(std_socket.try_clone()?)?;

        Ok(Self {
            stream,
            producer_id: self.producer_id.clone(),
        })
    }
}

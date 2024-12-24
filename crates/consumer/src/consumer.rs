use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum BrokerMessage {
    Publish {
        key: String,
        topic: String,
        payload: Vec<u8>,
    },
    Subscribe {
        consumer_id: String,
        topic: String,
    },
    Consume {
        consumer_id: String,
    },
    Register {
        client_id: String,
        client_type: String,
    },
    UpdateOffset {
        consumer_id: String,
        topic: String,
        offset: i64,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ConsumeResponse {
    message_id: String,
    topic: String,
    payload: Vec<u8>,
    sent_at: i64,
    offset: i64,
}

pub struct Consumer {
    stream: TcpStream,
    consumer_id: String,
    current_offset: i64,
}

impl Consumer {
    pub async fn new(addr: &str) -> Result<Self, Box<dyn Error>> {
        let stream = TcpStream::connect(addr).await?;
        let consumer_id = Uuid::new_v4().to_string();
        
        let mut consumer = Self {
            stream,
            consumer_id,
            current_offset: 0,
        };

        //reg
        let register_msg = BrokerMessage::Register {
            client_id: consumer.consumer_id.clone(),
            client_type: "consumer".to_string(),
        };
        
        consumer.send_message(&register_msg).await?;
        let _response = consumer.read_response().await?;
        
        println!("Consumer registered with ID: {}", consumer.consumer_id);
        Ok(consumer)
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

    pub async fn subscribe(&mut self, topic: String) -> Result<(), Box<dyn Error>> {
        let subscribe_msg = BrokerMessage::Subscribe {
            consumer_id: self.consumer_id.clone(),
            topic,
        };
        
        self.send_message(&subscribe_msg).await?;
        let response = self.read_response().await?;
        
        if response == "Subscribed successfully" {
            Ok(())
        } else {
            Err(response.into())
        }
    }

    pub async fn consume(&mut self, topic: String) -> Result<mpsc::Receiver<Vec<u8>>, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel(100);
        
        // Create a new connection for consuming messages
        let mut consume_stream = self.stream.try_clone().await?;
        
        // Send consume request
        let consume_msg = BrokerMessage::Consume {
            consumer_id: self.consumer_id.clone(),
        };
        
        self.send_message(&consume_msg).await?;

        // Spawn a task to continuously read messages
        let consumer_id = self.consumer_id.clone();
        let topic_clone = topic.clone();
        
        tokio::spawn(async move {
            let mut buffer = vec![0; 1024 * 64]; // 64KB buffer
            
            loop {
                match consume_stream.read(&mut buffer).await {
                    Ok(n) if n > 0 => {
                        if let Ok(message) = serde_json::from_slice::<ConsumeResponse>(&buffer[..n]) {
                            // Send the message payload to the channel
                            if tx.send(message.payload).await.is_err() {
                                break;
                            }
                            
                            // Send offset update
                            if let Ok(mut update_stream) = consume_stream.try_clone().await {
                                let update_msg = BrokerMessage::UpdateOffset {
                                    consumer_id: consumer_id.clone(),
                                    topic: topic_clone.clone(),
                                    offset: message.offset,
                                };
                                
                                if let Ok(msg_bytes) = serde_json::to_vec(&update_msg) {
                                    let _ = update_stream.write_all(&msg_bytes).await;
                                }
                            }
                        }
                    }
                    Ok(0) => break, // Connection closed
                    Err(_) => break, // Error occurred
                    _ => continue,
                }
            }
        });

        Ok(rx)
    }

    pub async fn update_offset(&mut self, topic: String, offset: i64) -> Result<(), Box<dyn Error>> {
        let update_msg = BrokerMessage::UpdateOffset {
            consumer_id: self.consumer_id.clone(),
            topic,
            offset,
        };
        
        self.send_message(&update_msg).await?;
        let response = self.read_response().await?;
        
        if response.contains("Offset updated") {
            self.current_offset = offset;
            Ok(())
        } else {
            Err(response.into())
        }
    }
}
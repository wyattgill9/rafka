use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

// Message types to replace gRPC messages
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
    GetMetrics,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ConsumeResponse {
    message_id: String,
    topic: String,
    payload: Vec<u8>,
    sent_at: i64,
    offset: i64,
}

pub struct Broker {
    topics: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    messages: Arc<RwLock<HashMap<u32, broadcast::Sender<ConsumeResponse>>>>,
    message_counter: AtomicUsize,
    broadcast_capacity: usize,
    partition_id: u32,
    total_partitions: u32,
    consumer_offsets: Arc<RwLock<HashMap<(String, String), i64>>>,
}

impl Broker {
    pub fn new(partition_id: u32, total_partitions: u32) -> Self {
        const BROADCAST_CAPACITY: usize = 1024 * 16;

        Self {
            topics: Arc::new(RwLock::new(HashMap::new())),
            messages: Arc::new(RwLock::new(HashMap::new())),
            message_counter: AtomicUsize::new(0),
            broadcast_capacity: BROADCAST_CAPACITY,
            partition_id,
            total_partitions,
            consumer_offsets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn handle_client(
        broker: Arc<Self>,
        mut socket: TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = vec![0; 1024 * 64]; // 64KB buffer

        loop {
            let n = socket.read(&mut buffer).await?;

            if n == 0 {
                break;
            }

            let message: BrokerMessage = serde_json::from_slice(&buffer[..n])?;

            match message {
                BrokerMessage::Publish {
                    key,
                    topic,
                    payload,
                } => {
                    if !broker.owns_partition(&key) {
                        let error = format!(
                            "Message belongs to partition {} not {}",
                            broker.hash_key(&key) % broker.total_partitions,
                            broker.partition_id
                        );
                        socket.write_all(error.as_bytes()).await?;
                        continue;
                    }

                    broker.ensure_topic(&topic).await;

                    let message_id = Uuid::new_v4().to_string();
                    let offset = broker.message_counter.fetch_add(1, Ordering::SeqCst) as i64;

                    let response = ConsumeResponse {
                        message_id: message_id.clone(),
                        topic: topic.clone(),
                        payload,
                        sent_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64,
                        offset,
                    };

                    let sender = broker.ensure_channel(broker.partition_id).await;
                    if let Err(e) = sender.send(response) {
                        eprintln!("Failed to broadcast message: {}", e);
                    }

                    let success = format!(
                        "Published to partition {} with offset {}",
                        broker.partition_id, offset
                    );
                    socket.write_all(success.as_bytes()).await?;
                }

                BrokerMessage::Subscribe { consumer_id, topic } => {
                    broker.ensure_topic(&topic).await;

                    let mut topics = broker.topics.write().await;
                    topics
                        .entry(topic.clone())
                        .or_insert_with(HashSet::new)
                        .insert(consumer_id.clone());

                    socket.write_all(b"Subscribed successfully").await?;
                }

                BrokerMessage::Consume { consumer_id } => {
                    let sender = broker.ensure_channel(broker.partition_id).await;
                    let mut rx = sender.subscribe();

                    // Spawn a task to handle this consumer
                    let std_socket: std::net::TcpStream = socket.into_std()?;
                    socket = TcpStream::from_std(std_socket.try_clone()?)?;

                    tokio::spawn(async move {
                        while let Ok(msg) = rx.recv().await {
                            if let Ok(data) = serde_json::to_vec(&msg) {
                                let mut tokio_clone = TcpStream::from_std(std_socket.try_clone()?)?;

                                if tokio_clone.write_all(&data).await.is_err() {
                                    break;
                                }
                            }
                        }

                        Ok::<(), std::io::Error>(())
                    });
                }

                BrokerMessage::UpdateOffset {
                    consumer_id,
                    topic,
                    offset,
                } => {
                    if offset < 0 {
                        socket.write_all(b"Offset cannot be negative").await?;
                        continue;
                    }

                    let topics = broker.topics.read().await;
                    if !topics.contains_key(&topic) {
                        socket.write_all(b"Topic not found").await?;
                        continue;
                    }

                    broker
                        .set_consumer_offset(&consumer_id, &topic, offset)
                        .await;
                    socket
                        .write_all(format!("Offset updated to {}", offset).as_bytes())
                        .await?;
                }

                _ => {
                    socket.write_all(b"Unsupported operation").await?;
                }
            }
        }

        Ok(())
    }

    pub async fn serve(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Broker listening on {}", addr);

        let broker = Arc::new(self);

        loop {
            let (socket, _) = listener.accept().await?;
            let broker = broker.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_client(broker, socket).await {
                    eprintln!("Error handling client: {}", e);
                }
            });
        }
    }

    async fn ensure_channel(&self, partition_id: u32) -> broadcast::Sender<ConsumeResponse> {
        let mut channels = self.messages.write().await;
        if let Some(sender) = channels.get(&partition_id) {
            if sender.receiver_count() > 0 {
                return sender.clone();
            }
        }

        let (new_tx, _) = broadcast::channel(self.broadcast_capacity);
        channels.insert(partition_id, new_tx.clone());
        new_tx
    }

    fn owns_partition(&self, message_key: &str) -> bool {
        let hash = self.hash_key(message_key);
        hash % self.total_partitions == self.partition_id
    }

    fn hash_key(&self, key: &str) -> u32 {
        key.bytes().fold(0u32, |acc, b| acc.wrapping_add(b as u32))
    }

    async fn ensure_topic(&self, topic: &str) {
        let topics = self.topics.read().await;
        if !topics.contains_key(topic) {
            drop(topics);
            let mut topics = self.topics.write().await;
            if !topics.contains_key(topic) {
                topics.insert(topic.to_string(), HashSet::new());
            }
        }
    }

    async fn set_consumer_offset(&self, consumer_id: &str, topic: &str, offset: i64) {
        let mut offsets = self.consumer_offsets.write().await;
        offsets.insert((consumer_id.to_string(), topic.to_string()), offset);
    }
}

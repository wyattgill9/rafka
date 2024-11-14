use as_any::Downcast;
use async_trait::async_trait;
use my_core::errors::broker_error::BrokerError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

use my_core::broker::Broker;
use my_core::message::Message;
use my_core::storage_engine::StorageEngine;

use crate::hooks_integration::{HookRegistry as BrokerHookRegistry, HookResultType};
use crate::request::Request;
use crate::response::Response;

type ProducerId = String;
type ConsumerId = String;

/// The `SimpleBroker` struct implements the broker logic, managing producers, consumers, and message dispatching.
pub struct SimpleBroker<S> {
    storage: Arc<S>,
    producers: Arc<Mutex<HashMap<ProducerId, mpsc::Sender<Box<dyn Message>>>>>,
    consumers: Arc<Mutex<HashMap<ConsumerId, mpsc::Receiver<Box<dyn Message>>>>>,
    hook_registry: Arc<BrokerHookRegistry>,
}

impl<S> SimpleBroker<S> {
    /// Creates a new `SimpleBroker` instance.
    pub fn new(storage: Arc<S>, hook_registry: Arc<BrokerHookRegistry>) -> Self {
        Self {
            storage,
            producers: Arc::new(Mutex::new(HashMap::new())),
            consumers: Arc::new(Mutex::new(HashMap::new())),
            hook_registry,
        }
    }
}

impl<S> SimpleBroker<S>
where
    S: StorageEngine,
{
    async fn handle_request(&self, request: Request) -> Result<Response, BrokerError> {
        // Invoke OnRequestReceived hooks
        for hook in &self.hook_registry.on_request_received_hooks {
            match hook.on_request_received(&request) {
                HookResultType::Continue => {}
                HookResultType::StopProcessing => {
                    return Err(BrokerError::HookError("Processing stopped by hook".into()))
                }
                HookResultType::ModifyRequest(modified_requestx) => {
                    // Use modified_request
                }
                HookResultType::Error(err) => return Err(BrokerError::HookError(err.message)),
                _ => {}
            }
        }

        // Process the request
        let response = self.process_request(&request).await?;

        // Invoke OnRequestExecuted hooks
        for hook in &self.hook_registry.on_request_executed_hooks {
            match hook.on_request_executed(&request, &response) {
                HookResultType::Continue => {}
                HookResultType::StopProcessing => {
                    return Err(BrokerError::HookError("Processing stopped by hook".into()))
                }
                HookResultType::ModifyResponse(modified_response) => {
                    // Use modified_response
                }
                HookResultType::Error(err) => return Err(BrokerError::HookError(err.message)),
                _ => {}
            }
        }

        Ok(response)
    }

    /// Processes an incoming request and generates a response.
    ///
    /// This is a placeholder implementation and should be customized based on actual requirements.
    async fn process_request(&self, request: &Request) -> Result<Response, BrokerError> {
        // Placeholder: Echo the request body in the response
        let response = Response::new(
            200,
            vec![("Content-Type".into(), "application/octet-stream".into())],
            request.body.clone(),
        );

        Ok(response)
    }
}

#[async_trait]
impl<S> Broker for SimpleBroker<S>
where
    S: StorageEngine + 'static,
{
    type ProducerId = ProducerId;
    type ConsumerId = ConsumerId;

    /// Registers a producer with the broker.
    ///
    /// Creates a channel for the producer to send messages to the broker.
    async fn register_producer(&self, producer_id: Self::ProducerId) -> Result<(), BrokerError> {
        let mut producers = self.producers.lock().await;
        if producers.contains_key(&producer_id) {
            return Err(BrokerError::RegistrationFailed(format!(
                "Producer '{}' is already registered",
                producer_id
            )));
        }

        let (tx, mut rx) = mpsc::channel(100);
        producers.insert(producer_id.clone(), tx);

        let storage = self.storage.clone();
        let hook_registry = self.hook_registry.clone();

        // Spawn a task to handle incoming messages from the producer
        let handle = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                // Invoke hooks before storing the message (omitted for brevity)
                // ...

                // Store the message
                if let Err(err) = storage
                    .append(
                        if let Some(msg) = message
                            .as_any()
                            .downcast_ref::<<S as StorageEngine>::Msg>()
                            .cloned()
                        {
                            msg
                        } else {
                            panic!("Error, {:?} cannot be downcasted!", message)
                        },
                    )
                    .await
                {
                    // Log or handle storage errors
                    eprintln!("Storage error: {:?}", err);
                }
            }
        });

        handle.await.unwrap();

        Ok(())
    }

    /// Registers a consumer with the broker.
    ///
    /// Creates a channel for the broker to send messages to the consumer.
    async fn register_consumer(&self, consumer_id: Self::ConsumerId) -> Result<(), BrokerError> {
        let mut consumers = self.consumers.lock().await;
        if consumers.contains_key(&consumer_id) {
            return Err(BrokerError::RegistrationFailed(format!(
                "Consumer '{}' is already registered",
                consumer_id
            )));
        }

        let (tx, rx) = mpsc::channel(100);
        consumers.insert(consumer_id.clone(), rx);

        // Additional setup for consumer message handling can be added here

        Ok(())
    }

    /// Dispatches a message to all registered consumers and stores it using the storage engine.
    async fn dispatch_message(&self, message: Box<dyn Message>) -> Result<(), BrokerError> {
        // Invoke hooks before dispatching (omitted for brevity)
        // ...

        // Store the message
        self.storage
            .append(
                if let Some(msg) = message
                    .as_any()
                    .downcast_ref::<<S as StorageEngine>::Msg>()
                    .cloned()
                {
                    msg
                } else {
                    panic!("Error, {:?} cannot be downcasted!", message)
                },
            )
            .await?;

        // Dispatch the message to all consumers
        let consumers = self.consumers.lock().await;
        for (consumer_id, consumer_tx) in consumers.iter() {
            if let Err(e) = consumer_tx.send(message.clone()).await {
                // Handle send error (e.g., consumer might have disconnected)
                eprintln!(
                    "Failed to send message to consumer '{}': {:?}",
                    consumer_id, e
                );
                return Err(BrokerError::DispatchFailed(format!(
                    "Failed to send to consumer '{}': {}",
                    consumer_id, e
                )));
            }
        }

        Ok(())
    }
}

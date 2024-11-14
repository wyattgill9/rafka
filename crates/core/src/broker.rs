use std::{fmt::Debug, hash::Hash};

use crate::{errors::broker_error::BrokerError, message::Message};

/// Trait representing a message broker in the system.
///
/// Implementors can define custom types for producer and consumer IDs, as well as error handling.
#[async_trait::async_trait]
pub trait Broker {
    type ProducerId: Debug + Clone + PartialEq + Eq + Hash + Send + Sync;
    type ConsumerId: Debug + Clone + PartialEq + Eq + Hash + Send + Sync;

    /// Registers a producer with the broker.
    ///
    /// # Arguments
    ///
    /// * `producer_id` - The identifier of the producer.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn register_producer(&self, producer_id: Self::ProducerId) -> Result<(), BrokerError>;

    /// Registers a consumer with the broker.
    ///
    /// # Arguments
    ///
    /// * `consumer_id` - The identifier of the consumer.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn register_consumer(&self, consumer_id: Self::ConsumerId) -> Result<(), BrokerError>;

    /// Dispatches a message to the appropriate consumers.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be dispatched.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn dispatch_message(&self, message: Box<dyn Message>) -> Result<(), BrokerError>;
}

use crate::{errors::consumer_error::ConsumerError, message::Message};

/// Trait representing a consumer in the messaging system.
///
/// Implementors define how messages are polled and committed.
/// The associated types `Msg` and `Error` allow for flexibility in message and error handling.
#[async_trait::async_trait]
pub trait Consumer {
    type Msg: Message;

    /// Polls for the next available message.
    ///
    /// This method should be called repeatedly to receive messages from the broker.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Self::Msg))` - A message was successfully received.
    /// * `Ok(None)` - No message is currently available; try again later.
    /// * `Err(Self::Error)` - An error occurred while polling for messages.
    async fn poll(&mut self) -> Result<Option<Self::Msg>, ConsumerError>;

    /// Commits the offset of the last consumed message.
    ///
    /// This method acknowledges to the broker that messages up to the current point have been processed.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The commit was successful.
    /// * `Err(Self::Error)` - An error occurred during the commit operation.
    async fn commit(&self) -> Result<(), ConsumerError>;
}

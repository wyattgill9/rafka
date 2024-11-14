use crate::{message::Message, ProducerError};

/// Trait representing a producer in the messaging system.
///
/// Implementors define how messages are sent to the broker or messaging system.
/// The associated types `Msg` and `Error` allow for flexibility in message and error handling.
#[async_trait::async_trait]
pub trait Producer {
    /// The type of messages produced.
    ///
    /// Must implement the `Message` trait.
    type Msg: Message;

    /// Sends a message to the broker or messaging system.
    ///
    /// This method may buffer the message internally and return immediately.
    /// To ensure all buffered messages are sent, call `flush`.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to be sent.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - The message was accepted for sending.
    /// * `Err(Self::Error)` - An error occurred while sending the message.
    async fn send(&self, message: Self::Msg) -> Result<(), ProducerError>;

    /// Flushes any buffered messages, ensuring they are sent.
    ///
    /// This method blocks until all pending messages have been sent or an error occurs.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All buffered messages were successfully sent.
    /// * `Err(Self::Error)` - An error occurred while flushing messages.
    async fn flush(&self) -> Result<(), ProducerError>;
}

use thiserror::Error;

/// Error type for the `SimpleProducer`, using `thiserror` for better error handling.
#[derive(Debug, Error)]
pub enum ProducerError {
    #[error("Broker error: {0}")]
    BrokerError(String),

    #[error("Send error: {0}")]
    SendError(String),

    #[error("Flush error: {0}")]
    FlushError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

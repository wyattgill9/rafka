use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConsumerError {
    #[error("Broker error: {0}")]
    BrokerError(String),

    #[error("Receive error: {0}")]
    ReceiveError(String),

    #[error("Commit error: {0}")]
    CommitError(String),
}

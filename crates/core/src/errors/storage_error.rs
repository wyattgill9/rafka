use thiserror::Error;

/// Error type for storage operations.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("I/O error occurred: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error occurred: {0}")]
    SerializationError(#[from] bincode::Error),

    #[error("Message not found: {0}")]
    NotFound(String),

    #[error("RocksDB error occurred: {0}")]
    RocksDbError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

use thiserror::Error;

use crate::StorageError;

/// Error type for the `SimpleBroker`, using `thiserror` for better error handling.
#[derive(Debug, Error)]
pub enum BrokerError {
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    #[error("Dispatch failed: {0}")]
    DispatchFailed(String),

    #[error("Hook error: {0}")]
    HookError(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] StorageError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

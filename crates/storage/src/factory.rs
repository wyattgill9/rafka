use std::sync::Arc;

use my_core::message::Message;
use my_core::storage_engine::StorageEngine;

use crate::in_memory::InMemoryStorage;
use my_core::errors::storage_error::StorageError;

use serde::Deserialize;

/// Configuration for the storage engine.
#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    pub storage_type: String,
    pub path: Option<String>,
}

/// Creates a storage engine based on the provided configuration.
///
/// # Arguments
///
/// * `config` - A reference to the storage configuration.
///
/// # Returns
///
/// An `Arc` to a storage engine implementing `StorageEngine<Msg = M>`.
pub fn create_storage_engine<M>(
    config: &StorageConfig,
) -> Result<Arc<dyn StorageEngine<Msg = M>>, StorageError>
where
    M: Message + 'static + Copy,
{
    match config.storage_type.as_str() {
        "in_memory" => Ok(Arc::new(InMemoryStorage::<M>::new())),

        other => Err(StorageError::InvalidConfig(format!(
            "Unsupported storage type '{}'",
            other
        ))),
    }
}

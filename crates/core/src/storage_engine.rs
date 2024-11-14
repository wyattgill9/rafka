use crate::errors::storage_error::StorageError;
use crate::message::Message;

#[async_trait::async_trait]
pub trait StorageEngine: Send + Sync {
    type Msg: Message + Clone;

    /// Append a message to the storage.
    async fn append(&self, message: Self::Msg) -> Result<(), StorageError>;

    /// Retrieve a message by offset.
    async fn get(&self, offset: u64) -> Result<Self::Msg, StorageError>;

    /// Retrieve a range of messages.
    async fn get_range(
        &self,
        start_offset: u64,
        end_offset: u64,
    ) -> Result<Vec<Self::Msg>, StorageError>;

    /// Get the latest offset.
    async fn latest_offset(&self) -> Result<u64, StorageError>;

    /// Perform a snapshot (if supported).
    async fn snapshot(&self) -> Result<(), StorageError>;

    /// Perform a recovery from WAL or snapshot (if supported).
    async fn recover(&self) -> Result<(), StorageError>;
}

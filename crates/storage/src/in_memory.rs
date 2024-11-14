// crates/storage/src/in_memory.rs

use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use my_core::message::Message;
use my_core::storage_engine::StorageEngine;

use my_core::errors::storage_error::StorageError;

/// In-memory storage engine with write-ahead log (WAL) and snapshot capabilities.
///
/// This implementation is primarily for testing and development purposes.
/// It stores messages in memory and supports snapshot and recovery operations.
pub struct InMemoryStorage<M> {
    log: Arc<RwLock<Vec<M>>>,
    snapshot: Arc<RwLock<Vec<M>>>,
    wal: Arc<Mutex<Vec<M>>>,
}

impl<M> InMemoryStorage<M> {
    /// Creates a new `InMemoryStorage` instance.
    pub fn new() -> Self {
        Self {
            log: Arc::new(RwLock::new(Vec::new())),
            snapshot: Arc::new(RwLock::new(Vec::new())),
            wal: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<M> InMemoryStorage<M>
where
    M: Message + Clone + 'static,
{
    /// Internal method to write a message to the write-ahead log (WAL).
    async fn write_to_wal(&self, message: &M) -> Result<(), StorageError> {
        let mut wal = self.wal.lock().await;
        wal.push(message.clone());
        Ok(())
    }

    /// Internal method to clear the write-ahead log (WAL) after a snapshot.
    async fn clear_wal(&self) {
        let mut wal = self.wal.lock().await;
        wal.clear();
    }
}

#[async_trait]
impl<M> StorageEngine for InMemoryStorage<M>
where
    M: Message + Clone + 'static,
{
    type Msg = M;

    /// Appends a message to the storage.
    ///
    /// The message is written to the WAL and then appended to the in-memory log.
    async fn append(&self, message: Self::Msg) -> Result<(), StorageError> {
        // Write to WAL
        self.write_to_wal(&message).await?;

        // Append to log
        let mut log = self.log.write().await;
        log.push(message);

        Ok(())
    }

    /// Retrieves a message by its offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - The offset of the message to retrieve.
    ///
    /// # Returns
    ///
    /// * `Ok(Self::Msg)` - The message at the specified offset.
    /// * `Err(StorageError::NotFound)` - If the message does not exist.
    async fn get(&self, offset: u64) -> Result<Self::Msg, StorageError> {
        let log = self.log.read().await;
        if let Some(msg) = log.get(offset as usize) {
            Ok(msg.clone())
        } else {
            Err(StorageError::NotFound(format!(
                "No message at offset {}",
                offset
            )))
        }
    }

    /// Retrieves a range of messages between `start_offset` (inclusive) and `end_offset` (exclusive).
    ///
    /// # Arguments
    ///
    /// * `start_offset` - The starting offset.
    /// * `end_offset` - The ending offset.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Self::Msg>)` - A vector of messages within the specified range.
    /// * `Err(StorageError)` - If an error occurs.
    async fn get_range(
        &self,
        start_offset: u64,
        end_offset: u64,
    ) -> Result<Vec<Self::Msg>, StorageError> {
        let log = self.log.read().await;
        let start = start_offset as usize;
        let end = end_offset as usize;

        if start >= log.len() {
            return Ok(Vec::new());
        }

        let end = std::cmp::min(end, log.len());
        Ok(log[start..end].to_vec())
    }

    /// Gets the latest offset (number of messages stored).
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - The latest offset.
    /// * `Err(StorageError)` - If an error occurs.
    async fn latest_offset(&self) -> Result<u64, StorageError> {
        let log = self.log.read().await;
        Ok(log.len() as u64)
    }

    /// Performs a snapshot of the current state.
    ///
    /// This method copies the current log to the snapshot and clears the WAL.
    async fn snapshot(&self) -> Result<(), StorageError> {
        let log = self.log.read().await;
        let mut snapshot = self.snapshot.write().await;
        *snapshot = log.clone();

        // Clear WAL after snapshot
        self.clear_wal().await;

        Ok(())
    }

    /// Recovers the state from the snapshot and applies any entries in the WAL.
    ///
    /// This method restores the log from the snapshot and replays the WAL entries.
    async fn recover(&self) -> Result<(), StorageError> {
        // Restore from snapshot
        let snapshot = self.snapshot.read().await;
        let mut log = self.log.write().await;
        *log = snapshot.clone();

        // Reapply WAL entries
        let wal = self.wal.lock().await;
        for message in wal.iter() {
            log.push(message.clone());
        }

        Ok(())
    }
}

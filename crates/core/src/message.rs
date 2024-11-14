use std::fmt::Debug;

use as_any::AsAny;
use dyn_clone::DynClone;

/// Trait representing a message in the messaging system.
///
/// Implementors define how message data and metadata are accessed.
/// The `Message` trait requires implementors to provide a payload and timestamp,
/// and to be thread-safe (`Send + Sync`).
pub trait Message: Send + Sync + DynClone + AsAny + Debug {
    /// Returns a reference to the payload of the message.
    ///
    /// # Returns
    ///
    /// A byte slice representing the message's payload.
    fn payload(&self) -> &[u8];

    /// Returns the timestamp of the message.
    ///
    /// # Returns
    ///
    /// A `u64` representing the timestamp, typically in milliseconds since the epoch.
    fn timestamp(&self) -> u64;
}

dyn_clone::clone_trait_object!(Message);

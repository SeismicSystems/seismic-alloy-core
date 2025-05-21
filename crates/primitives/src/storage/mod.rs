//! Seismic's storage

pub mod flagged_storage;
pub mod storage_slot;

pub use flagged_storage::FlaggedStorage;
pub use storage_slot::{PrivateSlot, StorageSlot};

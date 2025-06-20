//! Storage slot trait

use crate::{storage::FlaggedStorage, U256};

/// A word of data that can be stored in a storage slot
pub trait StorageSlot:
    Send + Sync + Default + Copy + Clone + Eq + PartialEq + PartialOrd + Ord + core::fmt::Debug
{
    /// The underlying value in the storage slot
    fn value(self) -> U256;
    /// The Below should return zeroed out, default words
    fn zero() -> Self {
        Self::default()
    }
    /// checks if the storage slot equals the zero() value
    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

impl StorageSlot for U256 {
    #[inline(always)]
    fn value(self) -> U256 {
        self
    }
}

/// Extends the `StorageSlot` trait to include a privacy flag
pub trait PrivateSlot: StorageSlot {
    /// whether the slot is private storage
    fn is_private(&self) -> bool;
}

impl StorageSlot for FlaggedStorage {
    #[inline(always)]
    fn value(self) -> U256 {
        self.value
    }
}

impl PrivateSlot for FlaggedStorage {
    fn is_private(&self) -> bool {
        self.is_private
    }
}

//! Abstraction for ethereum storage slots
//! Particularly to enable a privacy flag
#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;
use ruint::UintTryFrom;

use crate::{FixedBytes, U256};

/// A storage value that can be either private or public.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlaggedStorage {
    /// The value of the storage.
    pub value: U256,
    /// Whether the storage is private.
    pub is_private: bool,
}

impl<T> From<T> for FlaggedStorage
where
    U256: UintTryFrom<T>,
{
    fn from(value: T) -> Self {
        Self { value: U256::from(value), is_private: false }
    }
}

impl From<FlaggedStorage> for FixedBytes<32> {
    fn from(storage: FlaggedStorage) -> FixedBytes<32> {
        FixedBytes::<32>::from(storage.value)
    }
}

impl From<FlaggedStorage> for U256 {
    fn from(storage: FlaggedStorage) -> U256 {
        storage.value
    }
}

impl From<&FlaggedStorage> for U256 {
    fn from(storage: &FlaggedStorage) -> U256 {
        storage.value
    }
}

impl FlaggedStorage {
    /// The default word for a flagged storage slot
    /// when no state has been set. Importantly, this slot is public by default
    pub const ZERO: Self = Self { value: U256::ZERO, is_private: false };

    /// Create a new FlaggedStorage value from a given value and visibility.
    pub fn new<T>(value: T, is_private: bool) -> Self
    where
        U256: UintTryFrom<T>,
    {
        Self { value: U256::from(value), is_private }
    }

    /// Create a new FlaggedStorage value from a tuple of (value, is_private).
    pub fn new_from_tuple<T>((value, is_private): (T, bool)) -> Self
    where
        U256: UintTryFrom<T>,
    {
        Self { value: U256::from(value), is_private }
    }

    /// Create a new FlaggedStorage value from a given value.
    pub fn new_from_value<T>(value: T) -> Self
    where
        U256: UintTryFrom<T>,
    {
        Self {
            value: U256::from(value),
            is_private: false, // Default to false
        }
    }

    /// Collect the values from a HashMap of FlaggedStorage values.
    #[cfg(feature = "std")]
    pub fn collect_value<S: core::hash::BuildHasher + Default>(
        container: std::collections::HashMap<crate::B256, FlaggedStorage, S>,
    ) -> std::collections::HashMap<crate::B256, U256, S> {
        container.into_iter().map(|(key, flagged_storage)| (key, flagged_storage.value)).collect()
    }

    /// Check if the storage is private.
    pub fn is_private(&self) -> bool {
        self.is_private
    }

    /// Check if the storage is public.
    pub fn is_public(&self) -> bool {
        !self.is_private
    }

    /// Set the visibility of the storage.
    pub fn set_visibility(&self, is_private: bool) -> Self {
        FlaggedStorage { value: self.value, is_private }
    }

    /// Mark the storage as private.
    pub fn mark_private(&self) -> Self {
        self.set_visibility(true)
    }

    /// Mark the storage as public.
    pub fn mark_public(&self) -> Self {
        self.set_visibility(false)
    }

    /// Check if the storage is zero.
    pub fn is_zero(&self) -> bool {
        self.is_public() && self.value.is_zero()
    }
}

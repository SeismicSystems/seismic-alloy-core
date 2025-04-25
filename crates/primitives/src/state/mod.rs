use crate::{ruint::UintTryFrom, FixedBytes, U256};

#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;

#[cfg(feature = "map")]
use crate::{map::HashMap, B256};
#[cfg(feature = "map")]
use core::hash::{BuildHasher, Hash};

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A storage value with a flag to indicate whether it is private or public
pub struct FlaggedStorage {
    /// The underlying value
    pub value: U256,
    /// Whether the value is private or public
    pub is_private: bool,
}

impl From<U256> for FlaggedStorage {
    fn from(value: U256) -> Self {
        // by default, assume values are public (as original revm tests expect this)
        FlaggedStorage { value, is_private: false }
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
    /// The default value for a flagged storage slot
    /// when no state has been set. Importantly, this slot is public by default
    pub const ZERO: Self = Self { value: U256::ZERO, is_private: false };

    /// create a new flagged storage value
    pub fn new<T>(value: T, is_private: bool) -> Self
    where
        U256: UintTryFrom<T>,
    {
        Self { value: U256::from(value), is_private }
    }

    /// create a new flagged storage value from a tuple
    pub fn new_from_tuple<T>((value, is_private): (T, bool)) -> Self
    where
        U256: UintTryFrom<T>,
    {
        Self { value: U256::from(value), is_private }
    }

    /// create a new flagged storage value from a value
    /// defaults to private
    pub fn new_from_value<T>(value: T) -> Self
    where
        U256: UintTryFrom<T>,
    {
        Self {
            value: U256::from(value),
            is_private: false, // Default to false
        }
    }

    #[cfg(feature = "map")]
    /// Collects the values of the flagged storage into a `HashMap`
    pub fn collect_value<S: BuildHasher + Default>(
        container: HashMap<B256, FlaggedStorage, S>,
    ) -> HashMap<B256, U256, S> {
        container.into_iter().map(|(key, flagged_storage)| (key, flagged_storage.value)).collect()
    }

    /// returns whether the value is private
    pub fn is_private(&self) -> bool {
        self.is_private
    }

    /// returns whether the value is public
    pub fn is_public(&self) -> bool {
        !self.is_private
    }

    /// sets the visibility of the value
    pub fn set_visibility(&self, is_private: bool) -> Self {
        FlaggedStorage { value: self.value, is_private }
    }

    /// Sets the private flage to true
    pub fn mark_private(&self) -> Self {
        self.set_visibility(true)
    }

    /// sets the private flage to false
    pub fn mark_public(&self) -> Self {
        self.set_visibility(false)
    }

    /// Returns whether the value is the ZERO default value
    /// True if the value is zero and is flagged as publi
    pub fn is_zero(&self) -> bool {
        self.is_public() && self.value.is_zero()
    }
}

/// A trait to help make generic functions over flagged storage
/// Allows non-breaking changes to some function signatures to incorporate
/// flagged storage, which can be useful when code spans across multiple crates/repos
pub trait FlaggedStorageGeneric {
    /// returns whether the value is private
    fn is_private(&self) -> bool {
        false
    }
    /// returns the underlying value
    fn value(&self) -> &U256;
}

impl FlaggedStorageGeneric for U256 {
    fn value(&self) -> &Self {
        self
    }
}
impl FlaggedStorageGeneric for FlaggedStorage {
    fn is_private(&self) -> bool {
        self.is_private()
    }
    fn value(&self) -> &U256 {
        &self.value
    }
}

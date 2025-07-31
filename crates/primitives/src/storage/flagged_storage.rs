//! Abstraction for ethereum storage slots
//! Particularly to enable a privacy flag
use ruint::UintTryFrom;

use crate::{FixedBytes, U256};

/// A storage value that can be either private or public.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "arbitrary", derive(proptest_derive::Arbitrary, derive_arbitrary::Arbitrary))]
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

    /// Compare FlaggedStorage == U256
    /// We do not impl PartialEq<U256> for FlaggedStorage
    /// because it ends up conflicting with other PartialEq<U256> impls
    pub fn equals_u256(&self, other: &U256) -> bool {
        self.value == *other && !self.is_private
    }
}

#[cfg(feature = "rlp")]
mod rlp {
    use super::{FlaggedStorage, U256};

    use alloy_rlp::{Decodable, Encodable, Result as RlpResult};
    use bytes::BufMut;

    impl Encodable for FlaggedStorage {
        #[inline]
        fn length(&self) -> usize {
            self.value.length() + self.is_private.length()
        }

        #[inline]
        fn encode(&self, out: &mut dyn BufMut) {
            self.value.encode(out);
            self.is_private.encode(out);
        }
    }

    impl Decodable for FlaggedStorage {
        #[inline]
        fn decode(buf: &mut &[u8]) -> RlpResult<Self> {
            let value = U256::decode(buf)?;
            let is_private = bool::decode(buf)?;
            Ok(Self { value, is_private })
        }
    }

    use alloy_rlp::{MaxEncodedLen, MaxEncodedLenAssoc};
    // SAFETY: Assumes U256 and bool both have fixed max encoded lengths
    unsafe impl
        MaxEncodedLen<
            {
                <U256 as MaxEncodedLenAssoc>::LEN + 1 // bool encodes to 1 byte
            },
        > for FlaggedStorage
    {
    }

    unsafe impl MaxEncodedLenAssoc for FlaggedStorage {
        const LEN: usize = <U256 as MaxEncodedLenAssoc>::LEN + 1;
    }

    #[test]
    fn rlp_encode_decode() {
        let buf = &mut vec![];
        let flagged_a = FlaggedStorage::new(U256::from(1), false);
        flagged_a.encode(buf);
        let decoded = FlaggedStorage::decode(&mut buf.as_slice()).unwrap();
        assert_eq!(flagged_a, decoded);

        let buf = &mut vec![];
        let flagged_b = FlaggedStorage::new(U256::from(1), true);
        flagged_b.encode(buf);
        let decoded = FlaggedStorage::decode(&mut buf.as_slice()).unwrap();
        assert_eq!(flagged_b, decoded);
    }
}

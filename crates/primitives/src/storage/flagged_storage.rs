//! Abstraction for ethereum storage slots
//! Particularly to enable a privacy flag
#[cfg(feature = "arbitrary")]
use proptest_derive::Arbitrary;
use ruint::UintTryFrom;

use crate::U256;
use core::fmt;

/// A storage value that can be either private or public.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlaggedStorage {
    /// The value of the storage.
    pub value: U256,
    /// Whether the storage is private.
    pub is_private: bool,
}

/// Converts a `U256` into a **public** `FlaggedStorage`.
///
/// This impl exists solely because upstream `alloy-genesis` (from crates.io) calls
/// `seismic-trie::storage_root_unhashed<T: Into<FlaggedStorage>>` with `U256` values,
/// due to the `[patch.crates-io]` replacing `alloy-trie` with `seismic-trie`.
/// The Seismic codebase never actually calls this code path (we use `seismic-alloy-genesis`
/// which passes `FlaggedStorage` directly), but it still gets compiled as a transitive dependency.
///
/// TODO(samlaf): We might implement a refactor that would allow us to get rid of our seismic-trie
/// fork. See https://hackmd.io/@samlaf/SJcBaCBtbe). If we do implement this, then upstream alloy-genesis would
/// call upstream alloy-trie (which takes U256 directly) and this implicit conversion would no
/// longer be needed. I would recommend we instead force callers to use the explicit
/// `FlaggedStorage::public/private` instead.
impl From<U256> for FlaggedStorage {
    fn from(value: U256) -> Self {
        Self { value, is_private: false }
    }
}

impl FlaggedStorage {
    /// The default word for a flagged storage slot
    /// when no state has been set. Importantly, this slot is public by default
    pub const ZERO: Self = Self { value: U256::ZERO, is_private: false };

    /// Create a public flagged storage value
    pub fn public<T>(value: T) -> Self
    where
        U256: UintTryFrom<T>,
    {
        Self::new(value, false)
    }

    /// Create a private flagged storage value
    pub fn private<T>(value: T) -> Self
    where
        U256: UintTryFrom<T>,
    {
        Self::new(value, true)
    }

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

    const fn same_private(&self, other: bool) -> bool {
        self.is_private == other
    }

    /// Same as == but with references
    pub const fn const_eq(&self, other: &Self) -> bool {
        self.value.const_eq(&other.value) && self.same_private(other.is_private)
    }

    /// Same as == FlaggedStorage::ZERO
    pub const fn const_is_zero(&self) -> bool {
        self.const_eq(&FlaggedStorage::ZERO)
    }
}

impl fmt::Display for FlaggedStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_private {
            write!(f, "{} (private)", self.value)
        } else {
            write!(f, "{} (public)", self.value)
        }
    }
}

#[cfg(feature = "rlp")]
mod rlp {
    use super::{FlaggedStorage, U256};

    use alloy_rlp::{Decodable, Encodable, Result as RlpResult};
    use bytes::BufMut;

    /// RLP encoding for FlaggedStorage, used only for trie leaf value encoding
    /// (not for database storage, which uses Compact encoding via `StorageEntry`).
    ///
    /// Uses a varint-style scheme:
    /// - Public values encode identically to bare U256 (no extra bytes), so public storage produces
    ///   the same trie root as upstream Ethereum.
    /// - Private values append an extra `0x01` byte (RLP-encoded `true`) after the U256.
    ///
    /// Note that the 1 byte saving is purely for ethereum compatibility, and doesn't actually
    /// matter for performance. This RLP encoding is only used to compute trie roots, and these
    /// RLP encoded leaves are not stored to the DB. When writing to the DB, Compact encoding
    /// via `StorageEntry` is used instead. Also keccak256 pads in 136 byte blocks so the extra
    /// 1 byte really doesn't affect performance.
    impl Encodable for FlaggedStorage {
        fn length(&self) -> usize {
            self.value.length() + if self.is_private { 1 } else { 0 }
        }

        fn encode(&self, out: &mut dyn BufMut) {
            self.value.encode(out);
            if self.is_private {
                true.encode(out);
            }
        }
    }

    /// Decoding relies on buffer length: after decoding the U256, any remaining bytes
    /// indicate a private value. This means callers must pass exactly the leaf value
    /// bytes — a larger buffer with trailing data would be falsely decoded as private.
    // TODO(samlaf): should we assert the length then? Or that the value is 1? Or would that break
    // something?
    impl Decodable for FlaggedStorage {
        fn decode(buf: &mut &[u8]) -> RlpResult<Self> {
            let value = U256::decode(buf)?;
            let is_private = !buf.is_empty();
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
}

#[cfg(all(test, feature = "rlp"))]
mod rlp_tests {
    use super::{FlaggedStorage, U256};
    use alloy_rlp::{Decodable, Encodable};

    #[test]
    fn rlp_encode_decode() {
        // Public value encodes the same as bare U256
        let public_42 = FlaggedStorage::new(U256::from(42), false);
        let mut buf_public = vec![];
        public_42.encode(&mut buf_public);
        let mut buf_u256 = vec![];
        U256::from(42).encode(&mut buf_u256);
        assert_eq!(
            buf_public, buf_u256,
            "public FlaggedStorage should encode the same as bare U256"
        );

        // Roundtrip public
        let decoded = FlaggedStorage::decode(&mut buf_public.as_slice()).unwrap();
        assert_eq!(public_42, decoded);

        // Private value encodes as U256 bytes followed by bool(true) byte
        let private_42 = FlaggedStorage::new(U256::from(42), true);
        let mut buf_private = vec![];
        private_42.encode(&mut buf_private);
        let mut expected = vec![];
        U256::from(42).encode(&mut expected);
        true.encode(&mut expected);
        assert_eq!(
            buf_private, expected,
            "private FlaggedStorage should encode as U256 ++ bool(true)"
        );

        // Roundtrip private
        let decoded = FlaggedStorage::decode(&mut buf_private.as_slice()).unwrap();
        assert_eq!(private_42, decoded);

        // Public zero encodes the same as bare U256(0)
        let public_zero = FlaggedStorage::new(U256::ZERO, false);
        let mut buf_pub_zero = vec![];
        public_zero.encode(&mut buf_pub_zero);
        let mut buf_u256_zero = vec![];
        U256::ZERO.encode(&mut buf_u256_zero);
        assert_eq!(
            buf_pub_zero, buf_u256_zero,
            "public zero FlaggedStorage should encode the same as bare U256(0)"
        );
        println!("{}", buf_pub_zero.len());

        // Roundtrip public zero
        let decoded = FlaggedStorage::decode(&mut buf_pub_zero.as_slice()).unwrap();
        assert_eq!(public_zero, decoded);

        // Roundtrip private zero
        let private_zero = FlaggedStorage::new(U256::ZERO, true);
        let mut buf_priv_zero = vec![];
        private_zero.encode(&mut buf_priv_zero);
        let decoded = FlaggedStorage::decode(&mut buf_priv_zero.as_slice()).unwrap();
        assert_eq!(private_zero, decoded);
    }
}

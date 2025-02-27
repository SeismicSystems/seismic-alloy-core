//! Type aliases for common primitive types.

use crate::{FixedBytes, Signed, Uint};

pub use ruint::aliases::{U0, U1, U1024, U2048, U320, U384, U4096, U448};

macro_rules! int_aliases {
    ($($unsigned:ident, $signed:ident<$BITS:literal, $LIMBS:literal>),* $(,)?) => {$(
        #[doc = concat!($BITS, "-bit [unsigned integer type][Uint], consisting of ", $LIMBS, ", 64-bit limbs.")]
        pub type $unsigned = Uint<$BITS, $LIMBS>;

        #[doc = concat!($BITS, "-bit [signed integer type][Signed], consisting of ", $LIMBS, ", 64-bit limbs.")]
        pub type $signed = Signed<$BITS, $LIMBS>;

        const _: () = assert!($LIMBS == ruint::nlimbs($BITS));
    )*};
}

/// The 0-bit signed integer type, capable of representing 0.
pub type I0 = Signed<0, 0>;

/// The 1-bit signed integer type, capable of representing 0 and -1.
pub type I1 = Signed<1, 1>;

int_aliases! {
      U8,   I8<  8, 1>,
     U16,  I16< 16, 1>,
     U24,  I24< 24, 1>,
     U32,  I32< 32, 1>,
     U40,  I40< 40, 1>,
     U48,  I48< 48, 1>,
     U56,  I56< 56, 1>,
     U64,  I64< 64, 1>,

     U72,  I72< 72, 2>,
     U80,  I80< 80, 2>,
     U88,  I88< 88, 2>,
     U96,  I96< 96, 2>,
    U104, I104<104, 2>,
    U112, I112<112, 2>,
    U120, I120<120, 2>,
    U128, I128<128, 2>,

    U136, I136<136, 3>,
    U144, I144<144, 3>,
    U152, I152<152, 3>,
    U160, I160<160, 3>,
    U168, I168<168, 3>,
    U176, I176<176, 3>,
    U184, I184<184, 3>,
    U192, I192<192, 3>,

    U200, I200<200, 4>,
    U208, I208<208, 4>,
    U216, I216<216, 4>,
    U224, I224<224, 4>,
    U232, I232<232, 4>,
    U240, I240<240, 4>,
    U248, I248<248, 4>,
    U256, I256<256, 4>,

    U512, I512<512, 8>,
}

#[cfg(feature = "seismic")]
#[doc = "seismic unsigned integer type][Sint], where the preimage is a signed integer"]
#[derive(
    // Standard derives
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
)]
pub struct SUInt<const BITS: usize, const LIMBS: usize>(pub Uint<BITS, LIMBS>);

#[cfg(feature = "seismic")]
#[doc = "seismic unsigned integer type][Suint], where the preimage is an unsigned integer"]
#[derive(
    // Standard derives
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
)]
pub struct SInt<const BITS: usize, const LIMBS: usize>(pub Signed<BITS, LIMBS>);

#[cfg(feature = "seismic")]
macro_rules! sint_aliases {
    ($($unsigned:ident, $signed:ident<$BITS:literal, $LIMBS:literal>),* $(,)?) => {$(
        #[doc = concat!($BITS, "-bit [seismic unsigned integer type][Suint], where the preimage is an unsigned integer with ", $BITS, " bits.")]
        pub type $unsigned = SUInt<$BITS, $LIMBS>;

        #[doc = concat!($BITS, "-bit [seismic signed integer type][Sint], where the preimage is a signed integer with ", $BITS, " bits.")]
        pub type $signed = SInt<$BITS, $LIMBS>;
    )*};
}

#[cfg(feature = "seismic")]
#[derive(Copy, Clone, Debug, PartialEq)]
/// Seismic-shielded address type. Preimage is an address
pub struct SAddress(pub crate::Address);

#[cfg(all(feature = "seismic", feature = "arbitrary"))]
impl arbitrary::Arbitrary<'_> for SAddress {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let arbitrary_addr = u.arbitrary::<crate::Address>()?;
        Ok(SAddress(arbitrary_addr))
    }
}

#[cfg(all(feature = "seismic", feature = "arbitrary"))]
impl<const BITS: usize, const LIMBS: usize> arbitrary::Arbitrary<'_> for SUInt<BITS, LIMBS> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let arbitrary_uint = u.arbitrary::<Uint<BITS, LIMBS>>()?;
        Ok(SUInt(arbitrary_uint))
    }
}

#[cfg(all(feature = "seismic", feature = "arbitrary"))]
impl<const BITS: usize, const LIMBS: usize> arbitrary::Arbitrary<'_> for SInt<BITS, LIMBS> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> arbitrary::Result<Self> {
        let arbitrary_signed = u.arbitrary::<Signed<BITS, LIMBS>>()?;
        Ok(SInt(arbitrary_signed))
    }
}

#[cfg(feature = "seismic")]
sint_aliases! {
   SU8,   SI8<  8, 1>,
   SU16,  SI16< 16, 1>,
   SU24,  SI24< 24, 1>,
   SU32,  SI32< 32, 1>,
   SU40,  SI40< 40, 1>,
   SU48,  SI48< 48, 1>,
   SU56,  SI56< 56, 1>,
   SU64,  SI64< 64, 1>,

   SU72,  SI72< 72, 2>,
   SU80,  SI80< 80, 2>,
   SU88,  SI88< 88, 2>,
   SU96,  SI96< 96, 2>,
   SU104, SI104<104, 2>,
   SU112, SI112<112, 2>,
   SU120, SI120<120, 2>,
   SU128, SI128<128, 2>,

   SU136, SI136<136, 3>,
   SU144, SI144<144, 3  >,
   SU152, SI152<152, 3>,
   SU160, SI160<160, 3>,
   SU168, SI168<168, 3>,
   SU176, SI176<176, 3>,
   SU184, SI184<184, 3>,
   SU192, SI192<192, 3>,

   SU200, SI200<200, 4>,
   SU208, SI208<208, 4>,
   SU216, SI216<216, 4>,
   SU224, SI224<224, 4>,
   SU232, SI232<232, 4>,
   SU240, SI240<240, 4>,
   SU248, SI248<248, 4>,
   SU256, SI256<256, 4>,
}

macro_rules! fixed_bytes_aliases {
    ($($(#[$attr:meta])* $name:ident<$N:literal>),* $(,)?) => {$(
        #[doc = concat!($N, "-byte [fixed byte-array][FixedBytes] type.")]
        $(#[$attr])*
        pub type $name = FixedBytes<$N>;
    )*};
}

fixed_bytes_aliases! {
    B8<1>,
    B16<2>,
    B32<4>,
    B64<8>,
    B96<12>,
    B128<16>,
    /// See [`crate::B160`] as to why you likely want to use
    /// [`Address`](crate::Address) instead.
    #[doc(hidden)]
    B160<20>,
    B192<24>,
    B224<28>,
    B256<32>,
    B512<64>,
    B1024<128>,
    B2048<256>,
}

/// A block hash.
pub type BlockHash = B256;

/// A block number.
pub type BlockNumber = u64;

/// A block timestamp.
pub type BlockTimestamp = u64;

/// A transaction hash is a keccak hash of an RLP encoded signed transaction.
#[doc(alias = "TransactionHash")]
pub type TxHash = B256;

/// The sequence number of all existing transactions.
#[doc(alias = "TransactionNumber")]
pub type TxNumber = u64;

/// The nonce of a transaction.
#[doc(alias = "TransactionNonce")]
pub type TxNonce = u64;

/// The index of transaction in a block.
#[doc(alias = "TransactionIndex")]
pub type TxIndex = u64;

/// Chain identifier type (introduced in EIP-155).
pub type ChainId = u64;

/// An account storage key.
pub type StorageKey = B256;

/// An account storage value.
pub type StorageValue = U256;

/// Solidity contract functions are addressed using the first four bytes of the
/// Keccak-256 hash of their signature.
pub type Selector = FixedBytes<4>;

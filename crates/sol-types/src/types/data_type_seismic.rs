//! Seismic Solidity types.
use crate::types::data_type::{IntBitCount, Sealed};
use crate::{abi::token::WordToken, private::SolTypeValue, SolType, Word};
use alloc::vec::Vec;
use alloy_primitives::aliases::{SAddress as RustSAddress, *};
use core::{borrow::Borrow, fmt::*, hash::Hash, ops::*};

/// Saddress - `saddress`
#[derive(Clone, Copy, Debug)]
pub struct Saddress;

impl<T: Borrow<RustSAddress>> SolTypeValue<Saddress> for T
// where
//     T: Borrow<<IntBitCount<256> as SupportedSint>::Suint>,
//     IntBitCount<256>: SupportedSint,
{
    #[inline]
    fn stv_to_tokens(&self) -> WordToken {
        IntBitCount::<256>::tokenize_int(*self.borrow())
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
        IntBitCount::<256>::encode_packed_to_uint(*self.borrow(), out);
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> Word {
        SolTypeValue::<Suint<256>>::stv_to_tokens(self).0
    }
}

impl SolType for Saddress {
    type RustType = RustSAddress;
    type Token<'a> = WordToken;

    const SOL_NAME: &'static str = "saddress";
    const ENCODED_SIZE: Option<usize> = Some(32);
    const PACKED_ENCODED_SIZE: Option<usize> = Some(32);

    #[inline]
    fn valid_token(_token: &Self::Token<'_>) -> bool {
        return true;
    }

    #[inline]
    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        let s = &token.0[0..];
        Self::RustType::from_be_bytes::<32>(s.try_into().unwrap())
    }
}

/// Seismic Shielded Signed Integer - `sintX`
#[derive(Debug)]
pub struct Sint<const BITS: usize>;

impl<T, const BITS: usize> SolTypeValue<Sint<BITS>> for T
where
    T: Borrow<<IntBitCount<BITS> as SupportedSint>::Sint>,
    IntBitCount<BITS>: SupportedSint,
{
    #[inline]
    fn stv_to_tokens(&self) -> WordToken {
        IntBitCount::<BITS>::tokenize_int(*self.borrow())
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
        IntBitCount::<BITS>::encode_packed_to_int(*self.borrow(), out);
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> Word {
        SolTypeValue::<Sint<BITS>>::stv_to_tokens(self).0
    }
}

impl<const BITS: usize> SolType for Sint<BITS>
where
    IntBitCount<BITS>: SupportedSint,
{
    type RustType = <IntBitCount<BITS> as SupportedSint>::Sint;
    type Token<'a> = WordToken;

    const SOL_NAME: &'static str = IntBitCount::<BITS>::SINT_NAME;
    const ENCODED_SIZE: Option<usize> = Some(32);
    const PACKED_ENCODED_SIZE: Option<usize> = Some(32);

    #[inline]
    fn valid_token(_token: &Self::Token<'_>) -> bool {
        return true;
    }

    #[inline]
    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        IntBitCount::<BITS>::detokenize_int(token)
    }
}

/// Seismic Shielded Unsigned Integer - `suintX`
#[derive(Debug)]
pub struct Suint<const BITS: usize>;

impl<const BITS: usize, T> SolTypeValue<Suint<BITS>> for T
where
    T: Borrow<<IntBitCount<BITS> as SupportedSint>::Suint>,
    IntBitCount<BITS>: SupportedSint,
{
    #[inline]
    fn stv_to_tokens(&self) -> WordToken {
        IntBitCount::<BITS>::tokenize_uint(*self.borrow())
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
        IntBitCount::<BITS>::encode_packed_to_uint(*self.borrow(), out);
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> Word {
        SolTypeValue::<Suint<BITS>>::stv_to_tokens(self).0
    }
}

impl<const BITS: usize> SolType for Suint<BITS>
where
    IntBitCount<BITS>: SupportedSint,
{
    type RustType = <IntBitCount<BITS> as SupportedSint>::Suint;
    type Token<'a> = WordToken;

    const SOL_NAME: &'static str = IntBitCount::<BITS>::SUINT_NAME;
    const ENCODED_SIZE: Option<usize> = Some(32);
    const PACKED_ENCODED_SIZE: Option<usize> = Some(32);

    #[inline]
    fn valid_token(_token: &Self::Token<'_>) -> bool {
        return true;
    }

    #[inline]
    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        IntBitCount::<BITS>::detokenize_uint(token)
    }
}

// Declares types with the same traits
// TODO: Add more traits
// TODO: Integrate `num_traits` (needs `ruint`)
macro_rules! declare_sint_types {
    ($($(#[$attr:meta])* type $name:ident;)*) => {$(
        $(#[$attr])*
        type $name: Sized + Copy + PartialOrd + Ord + Eq + Hash
            + Not + BitAnd + BitOr + BitXor
            + Add + Sub + Mul + Div + Rem
            + AddAssign + SubAssign + MulAssign + DivAssign + RemAssign
            + Debug + Display + LowerHex + UpperHex + Octal + Binary;
    )*};
}

/// Statically guarantees that a [`Int`] or [`Uint`] bit count is marked as
/// supported.
///
/// This trait is *sealed*: the list of implementors below is total.
///
/// Users do not have the ability to mark additional [`IntBitCount<N>`] values
/// as supported. Only `Int` and `Uint` with supported byte counts are
/// constructable.
pub trait SupportedSint: Sealed {
    declare_sint_types! {
        /// The signed integer Rust representation.
        type Sint;

        /// The unsigned integer Rust representation.
        type Suint;
    }

    /// The name of the `Int` type: `int<N>`
    const SINT_NAME: &'static str;

    /// The name of the `Uint` type: `uint<N>`
    const SUINT_NAME: &'static str;

    /// The number of bits in the integer: `BITS`
    ///
    /// Note that this is not equal to `Self::Int::BITS`.
    const BITS: usize;

    /// The number of bytes in the integer: `BITS / 8`
    const BYTES: usize = Self::BITS / 8;

    /// Tokenizes a signed integer.
    fn tokenize_int(int: Self::Sint) -> WordToken;
    /// Detokenizes a signed integer.
    fn detokenize_int(token: WordToken) -> Self::Sint;
    /// ABI-encode a signed integer in packed mode.
    fn encode_packed_to_int(int: Self::Sint, out: &mut Vec<u8>);

    /// Tokenizes an unsigned integer.
    fn tokenize_uint(uint: Self::Suint) -> WordToken;
    /// Detokenizes an unsigned integer.
    fn detokenize_uint(token: WordToken) -> Self::Suint;
    /// ABI-encode an unsigned integer in packed mode.
    fn encode_packed_to_uint(uint: Self::Suint, out: &mut Vec<u8>);
}

macro_rules! supported_sint {
    ($($n:literal => $i:ident, $u:ident;)+) => {$(
        impl SupportedSint for IntBitCount<$n> {
            type Sint = $i;
            type Suint = $u;

            const SINT_NAME: &'static str = concat!("sint", $n);
            const SUINT_NAME: &'static str = concat!("suint", $n);

            const BITS: usize = $n;

            sint_impls2!($i);
            suint_impls2!($u);
        }
    )+};
}

macro_rules! sint_impls {
    (@big_int $ity:ident) => {
        #[inline]
        fn tokenize_int(int: $ity) -> WordToken {
            Self::tokenize_uint(int)
        }

        #[inline]
        fn detokenize_int(token: WordToken) -> $ity {
            Self::detokenize_uint(token)
        }

        #[inline]
        fn encode_packed_to_int(int: $ity, out: &mut Vec<u8>) {
            Self::encode_packed_to_uint(int, out);
        }
    };
    (@big_uint $uty:ident) => {
        #[inline]
        fn tokenize_uint(uint: $uty) -> WordToken {
            let mut word = Word::ZERO;
            word[..].copy_from_slice(&uint.to_be_bytes::<32>()[..]);
            WordToken(word)
        }

        #[inline]
        fn detokenize_uint(token: WordToken) -> $uty {
            // zero out bits to ignore
            let s = &token.0[..];
            <$uty>::from_be_bytes::<32>(s.try_into().unwrap())
        }

        #[inline]
        fn encode_packed_to_uint(uint: $uty, out: &mut Vec<u8>) {
            out.extend_from_slice(&uint.to_be_bytes::<32>()[..]);
        }
    };
}

#[rustfmt::skip]
macro_rules! sint_impls2 {
    ($t:ident) => { sint_impls! { @big_int $t } };
}

#[rustfmt::skip]
macro_rules! suint_impls2 {
    ($t:ident) => { sint_impls! { @big_uint $t } };
}

supported_sint!(
      8 =>  SI8,    SU8;
     16 =>  SI16,  SU16;
     24 =>  SI24,  SU24;
     32 =>  SI32,  SU32;
     40 =>  SI40,  SU40;
     48 =>  SI48,  SU48;
     56 =>  SI56,  SU56;
     64 =>  SI64,  SU64;
     72 =>  SI72,  SU72;
     80 =>  SI80,  SU80;
     88 =>  SI88,  SU88;
     96 =>  SI96,  SU96;
    104 => SI104, SU104;
    112 => SI112, SU112;
    120 => SI120, SU120;
    128 => SI128, SU128;
    136 => SI136, SU136;
    144 => SI144, SU144;
    152 => SI152, SU152;
    160 => SI160, SU160;
    168 => SI168, SU168;
    176 => SI176, SU176;
    184 => SI184, SU184;
    192 => SI192, SU192;
    200 => SI200, SU200;
    208 => SI208, SU208;
    216 => SI216, SU216;
    224 => SI224, SU224;
    232 => SI232, SU232;
    240 => SI240, SU240;
    248 => SI248, SU248;
    256 => SI256, SU256;
);

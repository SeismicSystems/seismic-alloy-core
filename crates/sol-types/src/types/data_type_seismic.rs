use crate::types::data_type::{IntBitCount, Sealed};
use crate::{abi::token::*, private::SolTypeValue, utils, SolType, Word};
use alloc::vec::Vec;
use alloy_primitives::{FixedBytes as RustFixedBytes, U256};
use core::{borrow::Borrow, fmt::*, hash::Hash, ops::*};

/// Saddress - `saddress`
#[derive(Clone, Copy, Debug)]
pub struct Saddress;

impl<T: Borrow<[u8; 32]>> SolTypeValue<Saddress> for T {
    #[inline]
    fn stv_to_tokens(&self) -> WordToken {
        WordToken(RustFixedBytes::<32>::new(*self.borrow()))
    }

    #[inline]
    fn stv_abi_encode_packed_to(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.borrow());
    }

    #[inline]
    fn stv_eip712_data_word(&self) -> Word {
        SolTypeValue::<Saddress>::stv_to_tokens(self).0
    }
}

impl SolType for Saddress {
    type RustType = RustFixedBytes<32>;
    type Token<'a> = WordToken;

    const SOL_NAME: &'static str = "saddress";
    const ENCODED_SIZE: Option<usize> = Some(32);
    const PACKED_ENCODED_SIZE: Option<usize> = Some(20);

    #[inline]
    fn detokenize(token: Self::Token<'_>) -> Self::RustType {
        token.0.try_into().unwrap()
    }

    #[inline]
    fn valid_token(token: &Self::Token<'_>) -> bool {
        utils::check_zeroes(&token.0[..12])
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

macro_rules! supported_int {
    ($($n:literal => $i:ident, $u:ident;)+) => {$(
        impl SupportedSint for IntBitCount<$n> {
            type Sint = $i;
            type Suint = $u;

            const SINT_NAME: &'static str = concat!("sint", $n);
            const SUINT_NAME: &'static str = concat!("suint", $n);

            const BITS: usize = $n;

            int_impls2!($i);
            uint_impls2!($u);
        }
    )+};
}

macro_rules! int_impls {
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
            word[0..].copy_from_slice(&uint.to_be_bytes::<32>()[0..]);
            WordToken(word)
        }

        #[inline]
        fn detokenize_uint(token: WordToken) -> $uty {
            // zero out bits to ignore
            let s = &token.0[0..];
            <$uty>::from_be_bytes::<32>(s.try_into().unwrap())
        }

        #[inline]
        fn encode_packed_to_uint(uint: $uty, out: &mut Vec<u8>) {
            out.extend_from_slice(&uint.to_be_bytes::<32>()[0..]);
        }
    };
}

#[rustfmt::skip]
macro_rules! int_impls2 {
    ($t:ident) => { int_impls! { @big_int $t } };
}

#[rustfmt::skip]
macro_rules! uint_impls2 {
    ($t:ident) => { int_impls! { @big_uint $t } };
}

supported_int!(
      8 => U256, U256;
     16 => U256, U256;
     24 => U256, U256;
     32 => U256, U256;
     40 => U256, U256;
     48 => U256, U256;
     56 => U256, U256;
     64 => U256, U256;
     72 => U256, U256;
     80 => U256, U256;
     88 => U256, U256;
     96 => U256, U256;
    104 => U256, U256;
    112 => U256, U256;
    120 => U256, U256;
    128 => U256, U256;
    136 => U256, U256;
    144 => U256, U256;
    152 => U256, U256;
    160 => U256, U256;
    168 => U256, U256;
    176 => U256, U256;
    184 => U256, U256;
    192 => U256, U256;
    200 => U256, U256;
    208 => U256, U256;
    216 => U256, U256;
    224 => U256, U256;
    232 => U256, U256;
    240 => U256, U256;
    248 => U256, U256;
    256 => U256, U256;
);

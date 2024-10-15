use crate::{seismic_util::Encryptable, transaction::SeismicTransaction};
use alloy_consensus::{SignableTransaction, Signed};
use alloy_primitives::Signature;
use alloy_rlp::{Buf, Header, EMPTY_STRING_CODE};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Encodes a signed SeismicTransaction into the provided buffer.
///
/// # Parameters
/// - `tx`: A reference to the signed SeismicTransaction to be encoded.
/// - `out`: A mutable reference to the buffer where the encoded transaction will be written.
#[allow(dead_code)]
pub fn encode_2718_seismic_transaction<T>(
    tx: &Signed<SeismicTransaction<T>>,
    out: &mut dyn alloy_rlp::BufMut,
) where
    T: Encryptable
        + Debug
        + Clone
        + PartialEq
        + Eq
        + Send
        + Sync
        + 'static
        + Serialize
        + for<'de> Deserialize<'de>,
{
    tx.tx().tx.encode_with_signature(tx.signature(), out, false);
}

/// Returns the length of the RLP-encoded signed SeismicTransaction.
///
/// # Parameters
/// - `tx`: A reference to the signed SeismicTransaction whose encoded length is to be calculated.
///
/// # Returns
/// The length of the RLP-encoded signed SeismicTransaction.
#[allow(dead_code)]
pub fn encode_2718_len<T>(tx: &Signed<SeismicTransaction<T>>) -> usize
where
    T: Encryptable
        + Debug
        + Clone
        + PartialEq
        + Eq
        + Send
        + Sync
        + 'static
        + Serialize
        + for<'de> Deserialize<'de>,
{
    tx.tx().tx.encoded_len_with_signature(tx.signature(), false)
}

/// Decodes a signed SeismicTransaction from the provided buffer.
///
/// # Parameters
/// - `buf`: A mutable reference to the buffer containing the RLP-encoded signed SeismicTransaction.
///
/// # Returns
/// A Result containing the decoded Signed<SeismicTransaction> or an alloy_rlp::Error if decoding
/// fails.
#[allow(dead_code)]
pub fn decode_signed_seismic_tx<T>(
    buf: &mut &[u8],
) -> Result<Signed<SeismicTransaction<T>>, alloy_rlp::Error>
where
    T: Encryptable
        + Debug
        + Clone
        + PartialEq
        + Eq
        + Send
        + Sync
        + 'static
        + Serialize
        + for<'de> Deserialize<'de>,
{
    let mut h_decode = *buf;
    let h = Header::decode(&mut h_decode)?;
    *buf = h_decode;

    if buf.len() < h.payload_length {
        return Err(alloy_rlp::Error::InputTooShort);
    }

    buf.advance(1); // Skip tx type
    let tx = decode_signed_seismic_fields(buf)?;

    let bytes_consumed = h_decode.len() - buf.len();

    if bytes_consumed != h.payload_length && h_decode[0] > EMPTY_STRING_CODE {
        return Err(alloy_rlp::Error::UnexpectedLength);
    }

    Ok(tx)
}

/// Decodes the fields of a signed SeismicTransaction from the provided buffer.
///
/// # Parameters
/// - `buf`: A mutable reference to the buffer containing the RLP-encoded fields of the signed
///   SeismicTransaction.
///
/// # Returns
/// A Result containing the decoded Signed<SeismicTransaction> or an alloy_rlp::Error if decoding
/// fails.
#[allow(dead_code)]
pub fn decode_signed_seismic_fields<T>(
    buf: &mut &[u8],
) -> alloy_rlp::Result<Signed<SeismicTransaction<T>>>
where
    T: Encryptable
        + Debug
        + Clone
        + PartialEq
        + Eq
        + Send
        + Sync
        + 'static
        + Serialize
        + for<'de> Deserialize<'de>,
{
    let header = Header::decode(buf)?;
    if !header.list {
        return Err(alloy_rlp::Error::UnexpectedString);
    }

    let original_len = buf.len();

    let tx = SeismicTransaction::decode_fields(buf)?;
    let signature = Signature::decode_rlp_vrs(buf)?;

    let signed = tx.into_signed(signature);
    if buf.len() + header.payload_length != original_len {
        return Err(alloy_rlp::Error::ListLengthMismatch {
            expected: header.payload_length,
            got: original_len - buf.len(),
        });
    }

    Ok(signed)
}

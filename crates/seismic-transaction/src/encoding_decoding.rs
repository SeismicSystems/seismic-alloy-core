use crate::transaction::SeismicTransaction;
use alloy_consensus::{SignableTransaction, Signed};
use alloy_primitives::{keccak256, Signature};
use alloy_rlp::{Buf, Decodable, Header, EMPTY_STRING_CODE};

/// Encodes a signed SeismicTransaction into the provided buffer.
///
/// # Parameters
/// - `tx`: A reference to the signed SeismicTransaction to be encoded.
/// - `out`: A mutable reference to the buffer where the encoded transaction will be written.
#[allow(dead_code)]
pub fn encode_2718_seismic_transaction(
    tx: &Signed<SeismicTransaction>,
    out: &mut dyn alloy_rlp::BufMut,
) {
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
pub fn encode_2718_len(tx: &Signed<SeismicTransaction>) -> usize {
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
pub fn _decode_enveloped_typed_transaction(
    buf: &mut &[u8],
) -> Result<Signed<SeismicTransaction>, alloy_rlp::Error> {
    let mut h_decode = *buf;
    let h = Header::decode(&mut h_decode)?;
    *buf = h_decode;

    if buf.len() < h.payload_length {
        return Err(alloy_rlp::Error::InputTooShort);
    }

    buf.advance(1); // Skip tx type
    let tx = _decode_signed_seismic_fields(buf)?;

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
pub fn _decode_signed_seismic_fields(
    buf: &mut &[u8],
) -> alloy_rlp::Result<Signed<SeismicTransaction>> {
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

fn decode_enveloped_seismic_tx(data: &mut &[u8]) -> alloy_rlp::Result<Signed<SeismicTransaction>> {
    let original_encoding_without_header = *data;

    // let tx_type = *data.first().ok_or(alloy_rlp::Error::InputTooShort)?;
    // if tx_type != SeismicTransaction::transaction_type() {
    //     return Err(alloy_rlp::Error::Custom("Not a seismic transaction"));
    // }
    // data.advance(1);

    // decode the list header for the rest of the transaction
    let header = Header::decode(data)?;
    if !header.list {
        return Err(alloy_rlp::Error::Custom("typed tx fields must be encoded as a list"))
    }

    let remaining_len = data.len();

    // length of tx encoding = tx type byte (size = 1) + length of header + payload length
    let tx_length = 1 + header.length() + header.payload_length;

    let tx = SeismicTransaction::decode_fields(data)?;
    let signature = Signature::decode(data)?;

    let bytes_consumed = remaining_len - data.len();
    if bytes_consumed != header.payload_length {
        return Err(alloy_rlp::Error::UnexpectedLength)
    }

    let hash = keccak256(&original_encoding_without_header[..tx_length]);
    let signed = Signed::<SeismicTransaction>::new_unchecked(tx, signature, hash);
    Ok(signed)
}


pub fn decode_signed_seismic_tx(buf: &mut &[u8]) -> alloy_rlp::Result<Signed<SeismicTransaction>> {
    if buf.is_empty() {
        return Err(alloy_rlp::Error::InputTooShort)
    }

    // decode header
    let original_encoding = *buf;
    let header = Header::decode(buf)?;

    if !header.list {
        return Err(alloy_rlp::Error::UnexpectedString);
    }

    let remaining_len = buf.len();
    let tx = decode_enveloped_seismic_tx(buf)?;

    let bytes_consumed = remaining_len - buf.len();
    // because Header::decode works for single bytes (including the tx type), returning a
    // string Header with payload_length of 1, we need to make sure this check is only
    // performed for transactions with a string header
    if bytes_consumed != header.payload_length && original_encoding[0] > EMPTY_STRING_CODE {
        return Err(alloy_rlp::Error::UnexpectedLength)
    }

    Ok(tx)
}
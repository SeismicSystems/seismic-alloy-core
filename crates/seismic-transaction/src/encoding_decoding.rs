use crate::transaction::SeismicTransaction;
use alloy_consensus::{SignableTransaction, Signed};
use alloy_primitives::Signature;
use alloy_rlp::{Buf, Header, EMPTY_STRING_CODE};

#[allow(dead_code)]
pub fn encode_2718_seismic_transaction(
    tx: &Signed<SeismicTransaction>,
    out: &mut dyn alloy_rlp::BufMut,
) {
    tx.tx().base.encode_with_signature(tx.signature(), out, false);
}

#[allow(dead_code)]
pub fn encode_2718_len(tx: &Signed<SeismicTransaction>) -> usize {
    tx.tx().base.encoded_len_with_signature(tx.signature(), false)
}

#[allow(dead_code)]
pub fn decode_signed_seismic_tx(
    buf: &mut &[u8],
) -> Result<Signed<SeismicTransaction>, alloy_rlp::Error> {
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

#[allow(dead_code)]
pub fn decode_signed_seismic_fields(
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

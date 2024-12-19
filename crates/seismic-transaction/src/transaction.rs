use alloy_consensus::{SignableTransaction, Signed, Transaction};
use alloy_eips::{eip2930::AccessList, eip7702::SignedAuthorization};
use alloy_primitives::{keccak256, Bytes, ChainId, Signature, TxKind, B256, U256};
use alloy_rlp::{length_of_length, BufMut, Decodable, Encodable, Header};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

impl Transaction for SeismicTransactionRequest {
    fn chain_id(&self) -> Option<ChainId> {
        Some(ChainId::from(self.chain_id))
    }
    fn nonce(&self) -> u64 {
        self.nonce
    }
    fn gas_limit(&self) -> u128 {
        self.gas_limit.try_into().unwrap_or(u128::MAX)
    }
    fn gas_price(&self) -> Option<u128> {
        Some(self.gas_price.try_into().unwrap_or(u128::MAX))
    }
    fn to(&self) -> TxKind {
        self.kind
    }
    fn value(&self) -> U256 {
        self.value
    }
    fn input(&self) -> &[u8] {
        &self.seismic_input
    }

    fn max_fee_per_gas(&self) -> u128 {
        self.gas_price.try_into().unwrap_or(u128::MAX)
    }

    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        Some(self.gas_price.try_into().unwrap_or(u128::MAX))
    }
    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        None
    }

    fn priority_fee_or_price(&self) -> u128 {
        self.gas_price.try_into().unwrap_or(u128::MAX)
    }

    fn ty(&self) -> u8 {
        SeismicTransaction::TRANSACTION_TYPE
    }

    fn access_list(&self) -> Option<&AccessList> {
        None
    }

    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        None
    }

    fn authorization_list(&self) -> Option<&[SignedAuthorization]> {
        None
    }
}

impl Encodable for SeismicTransactionRequest {
    fn encode(&self, out: &mut dyn BufMut) {
        self.chain_id.encode(out);
        self.nonce.encode(out);
        self.gas_price.encode(out);
        self.gas_limit.encode(out);
        self.kind.encode(out);
        self.value.encode(out);
        self.seismic_input.encode(out);
    }

    fn length(&self) -> usize {
        self.chain_id.length()
            + self.nonce.length()
            + self.gas_price.length()
            + self.gas_limit.length()
            + self.kind.length()
            + self.value.length()
            + self.seismic_input.length()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Represents a request for a seismic transaction.
pub struct SeismicTransactionRequest {
    /// The nonce of the transaction
    pub nonce: u64,
    /// The gas price for the transaction
    pub gas_price: u128,
    /// The gas limit for the transaction
    pub gas_limit: u64,
    /// The kind of transaction (e.g., Call, Create)
    pub kind: TxKind,
    /// The value of the transaction
    pub value: U256,
    /// The optional chain ID for the transaction
    pub chain_id: u64,
    /// The input data for the transaction
    pub seismic_input: Bytes,
}

/// Represents a seismic transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeismicTransaction {
    /// The base transaction data.
    #[serde(flatten)]
    pub tx: SeismicTransactionRequest,
}

impl SeismicTransactionRequest {
    /// Encodes only the transaction's fields into the desired buffer, without a RLP header.
    pub(crate) fn encode_fields(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.chain_id.encode(out);
        self.nonce.encode(out);
        self.gas_price.encode(out);
        self.gas_limit.encode(out);
        self.kind.encode(out);
        self.value.encode(out);
        self.seismic_input.encode(out);
    }

    /// Calculates the length of the RLP-encoded transaction's fields
    /// secret_data is not included in the length calculation since
    /// it is not part of the transaction's signature and hence
    /// not RLP-encoded.
    pub(crate) fn fields_len(&self) -> usize {
        self.chain_id.length()
            + self.nonce.length()
            + self.gas_price.length()
            + self.gas_limit.length()
            + self.kind.length()
            + self.value.length()
            + self.seismic_input.length()
    }

    /// Encodes the transaction from RLP bytes, including the signature. This __does not__ encode a
    /// tx type byte or string header.
    ///
    /// This __does__ encode a list header and include a signature.
    pub(crate) fn encode_with_signature_fields(&self, signature: &Signature, out: &mut dyn BufMut) {
        let payload_length = self.fields_len() + signature.rlp_vrs_len();
        let header = Header { list: true, payload_length };
        header.encode(out);
        self.encode_fields(out);
        signature.write_rlp_vrs(out);
    }

    /// Encodes the transaction with the provided signature into the desired buffer.
    ///
    /// # Parameters
    /// - `signature`: The signature to be included in the encoded transaction.
    /// - `out`: The buffer where the encoded transaction will be written.
    /// - `with_header`: A boolean flag indicating whether to include the RLP header in the
    ///   encoding.
    ///
    /// If `with_header` is `true`, the encoded transaction will include the RLP header.
    /// If `with_header` is `false`, the encoded transaction will not include the RLP header.
    pub fn encode_with_signature(
        &self,
        signature: &Signature,
        out: &mut dyn BufMut,
        with_header: bool,
    ) {
        let payload_length = self.fields_len() + signature.rlp_vrs_len();
        if with_header {
            Header {
                list: false,
                payload_length: 1 + Header { list: true, payload_length }.length() + payload_length,
            }
            .encode(out);
        }
        out.put_u8(self.ty() as u8);
        self.encode_with_signature_fields(signature, out);
    }

    /// Returns what the encoded length should be, if the transaction were RLP encoded with the
    /// given signature, depending on the value of `with_header`.
    ///
    /// If `with_header` is `true`, the payload length will include the RLP header length.
    /// If `with_header` is `false`, the payload length will not include the RLP header length.
    pub fn encoded_len_with_signature(&self, signature: &Signature, with_header: bool) -> usize {
        // this counts the tx fields and signature fields
        let payload_length = self.fields_len() + signature.rlp_vrs_len();

        // this counts:
        // * tx type byte
        // * inner header length
        // * inner payload length
        let inner_payload_length =
            1 + Header { list: true, payload_length }.length() + payload_length;
        if with_header {
            Header { list: false, payload_length: inner_payload_length }.length()
                + inner_payload_length
        } else {
            inner_payload_length
        }
    }
}

impl SeismicTransactionRequest {
    /// Computes the hash of the transaction request.
    ///
    /// This function encodes the base transaction fields using RLP encoding,
    /// then computes the Keccak-256 hash of the encoded data.
    ///
    /// # Returns
    /// A `B256` hash representing the Keccak-256 hash of the RLP encoded transaction fields.
    pub fn hash(&self) -> B256 {
        B256::from_slice(&keccak256(alloy_rlp::encode(&self))[..])
    }

    /// Converts the transaction request into a signed transaction object
    /// without signing the secret data field so as to not leak the secret data.
    pub fn into_signed_without_secrets(self, signature: Signature) -> Signed<SeismicTransaction> {
        let mut buf = Vec::with_capacity(self.encoded_len_with_signature(&signature, false));
        self.encode_with_signature(&signature, &mut buf, false);
        let hash = keccak256(&buf);
        Signed::new_unchecked(SeismicTransaction { tx: self }, signature.with_parity_bool(), hash)
    }
}

impl SignableTransaction<Signature> for SeismicTransactionRequest {
    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.chain_id = chain_id;
    }

    fn encode_for_signing(&self, out: &mut dyn alloy_rlp::BufMut) {
        out.put_u8(SeismicTransaction::TRANSACTION_TYPE);
        Header { list: true, payload_length: self.fields_len() }.encode(out);
        self.encode(out)
    }

    fn payload_len_for_signature(&self) -> usize {
        let payload_length = self.length();
        1 + length_of_length(payload_length) + self.length()
    }

    fn into_signed(self, _signature: Signature) -> Signed<Self> {
        unimplemented!()
    }
}

impl SeismicTransaction {
    /// Seismic transaction type is 74
    pub const TRANSACTION_TYPE: u8 = 0x4A;

    pub(crate) fn decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            tx: SeismicTransactionRequest {
                chain_id: Decodable::decode(buf)?,
                nonce: Decodable::decode(buf)?,
                gas_price: Decodable::decode(buf)?,
                gas_limit: Decodable::decode(buf)?,
                kind: Decodable::decode(buf)?,
                value: Decodable::decode(buf)?,
                seismic_input: Decodable::decode(buf)?,
            },
        })
    }
}

impl SignableTransaction<Signature> for SeismicTransaction {
    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.tx.chain_id = chain_id;
    }

    fn encode_for_signing(&self, out: &mut dyn alloy_rlp::BufMut) {
        out.put_u8(SeismicTransaction::TRANSACTION_TYPE);
        Header { list: true, payload_length: self.tx.length() }.encode(out);
        self.tx.encode(out)
    }

    fn payload_len_for_signature(&self) -> usize {
        let payload_length = self.tx.length();
        1 + length_of_length(payload_length) + self.tx.length()
    }

    fn into_signed(self, signature: Signature) -> Signed<Self> {
        // Drop any v chain id value to ensure the signature format is correct at the time of
        // combination for an EIP-7702 transaction. V should indicate the y-parity of the
        // signature.
        let signature = signature.with_parity_bool();

        let mut buf = Vec::with_capacity(self.tx.encoded_len_with_signature(&signature, false));
        self.tx.encode_with_signature(&signature, &mut buf, false);
        let hash = keccak256(&buf);

        Signed::new_unchecked(self, signature, hash)
    }
}

impl Transaction for SeismicTransaction {
    fn chain_id(&self) -> Option<ChainId> {
        Some(ChainId::from(self.tx.chain_id))
    }
    fn nonce(&self) -> u64 {
        self.tx.nonce
    }
    fn gas_limit(&self) -> u128 {
        self.tx.gas_limit.try_into().unwrap_or(u128::MAX)
    }
    fn gas_price(&self) -> Option<u128> {
        Some(self.tx.gas_price.try_into().unwrap_or(u128::MAX))
    }
    fn to(&self) -> TxKind {
        self.tx.kind
    }
    fn value(&self) -> U256 {
        self.tx.value
    }
    fn input(&self) -> &[u8] {
        &self.tx.seismic_input
    }
    fn max_fee_per_gas(&self) -> u128 {
        self.tx.gas_price.try_into().unwrap_or(u128::MAX)
    }
    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        Some(self.tx.gas_price.try_into().unwrap_or(u128::MAX))
    }
    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        None
    }
    fn priority_fee_or_price(&self) -> u128 {
        self.tx.gas_price.try_into().unwrap_or(u128::MAX)
    }
    fn ty(&self) -> u8 {
        Self::TRANSACTION_TYPE
    }
    fn access_list(&self) -> Option<&AccessList> {
        None
    }
    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        None
    }
    fn authorization_list(&self) -> Option<&[SignedAuthorization]> {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction::{SeismicTransaction, SeismicTransactionRequest};
    use alloy_primitives::{Address, Bytes, U256};
    use std::str::FromStr;

    #[test]
    fn test_encoding_fields() {
        let tx = SeismicTransaction {
            tx: SeismicTransactionRequest {
                chain_id: 4u64,
                nonce: 2,
                gas_price: 1000000000,
                gas_limit: 100000,
                kind: Address::from_str("d3e8763675e4c425df46cc3b5c0f6cbdac396046").unwrap().into(),
                value: U256::from(1000000000000000u64),
                seismic_input: vec![1, 2, 3].into(),
            },
        };

        let mut encoded_tx = Vec::new();
        tx.tx.encode_fields(&mut encoded_tx);
        let alloy_encoding = format!("{:0x}", Bytes::from(encoded_tx));

        let reth_encoding = String::from("0x0402843b9aca00830186a094d3e8763675e4c425df46cc3b5c0f6cbdac39604687038d7ea4c6800083010203");
        assert_eq!(reth_encoding, alloy_encoding);
    }
}

use crate::types::SecretData;
use alloy_consensus::{SignableTransaction, Signed, Transaction};
use alloy_eips::{eip2930::AccessList, eip7702::SignedAuthorization};
use alloy_primitives::{keccak256, Bytes, ChainId, Signature, TxKind, B256, U256};
use alloy_rlp::{BufMut, Decodable, Encodable, Header};
use serde::{Deserialize, Serialize};

/// Represents the base structure of a Seismic Transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeismicTransactionBase {
    /// The chain ID of the transaction.
    pub chain_id: ChainId,
    /// The nonce of the transaction.
    pub nonce: u64,
    /// The recipient of the transaction.
    pub to: TxKind,
    /// The gas limit for the transaction.
    pub gas_limit: u128,
    /// The maximum fee per gas for the transaction.
    pub max_fee_per_gas: u128,
    /// The maximum priority fee per gas for the transaction.
    pub max_priority_fee_per_gas: u128,
    /// The value being transferred in the transaction.
    pub value: U256,
    /// The access list for the transaction.
    pub access_list: AccessList,
    /// The input data for the transaction.
    pub input: Bytes,
}

/// A trait representing a Seismic Transaction.
pub trait SeismicTx: Sized {
    /// Returns a reference to the base of the Seismic Transaction.
    fn base(&self) -> &SeismicTransactionBase;
    /// Returns a mutable reference to the base of the Seismic Transaction.
    fn base_mut(&mut self) -> &mut SeismicTransactionBase;
    /// Returns the transaction type.
    fn tx_type() -> u8 {
        0x64
    }
}

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
        &self.encrypted_input
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
        0x64
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
        self.nonce.encode(out);
        self.gas_price.encode(out);
        self.gas_limit.encode(out);
        self.kind.encode(out);
        self.value.encode(out);
        self.encrypted_input.encode(out);
        self.chain_id.encode(out);
    }

    fn length(&self) -> usize {
        self.nonce.length() +
        self.gas_price.length() +
        self.gas_limit.length() +
        self.kind.length() +
        self.value.length() +
        self.encrypted_input.length() +
        self.chain_id.length()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Represents a request for a seismic transaction.
pub struct SeismicTransactionRequest {
    /// The nonce of the transaction
    pub nonce: u64,
    /// The gas price for the transaction
    pub gas_price: U256,
    /// The gas limit for the transaction
    pub gas_limit: U256,
    /// The kind of transaction (e.g., Call, Create)
    pub kind: TxKind,
    /// The value of the transaction
    pub value: U256,
    /// The encrypted data for the transaction
    pub encrypted_input: Vec<u8>,
    /// The optional chain ID for the transaction
    pub chain_id: u64,
    // /// The base transaction data.
    // #[serde(flatten)]
    // pub base: SeismicTransactionBase,
    // /// A vector containing secret data associated with the transaction.
    // pub secret_data: Vec<SecretData>,
}

/// Represents a seismic transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeismicTransaction {
    /// The base transaction data.
    #[serde(flatten)]
    pub tx: SeismicTransactionRequest,
}


// impl Encodable for SeismicTransactionBase {
//     fn encode(&self, out: &mut dyn BufMut) {
//         let payload_length = self.fields_len();
//         Header { list: true, payload_length }.encode(out);
//         self.encode_fields(out);
//     }

//     fn length(&self) -> usize {
//         let payload_length = self.fields_len();
//         Header { list: true, payload_length }.length() + payload_length
//     }
// }

impl SeismicTransactionRequest {
    /// Encodes only the transaction's fields into the desired buffer, without a RLP header.
    pub(crate) fn encode_fields(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.chain_id.encode(out);
        self.nonce.encode(out);
        self.gas_price.encode(out);
        self.gas_limit.encode(out);
        self.kind.encode(out);
        self.value.encode(out);
        self.encrypted_input.encode(out);
        self.chain_id.encode(out);
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
            + self.encrypted_input.length()
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
        out.put_u8(self.chain_id as u8);
        self.encode(out)
    }

    fn payload_len_for_signature(&self) -> usize {
        1 + self.length()
    }

    fn into_signed(self, _signature: Signature) -> Signed<Self> {
        unimplemented!()
    }
}

impl SeismicTransaction {
    pub(crate) fn decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self {
            tx: SeismicTransactionRequest {
                chain_id: Decodable::decode(buf)?,
                nonce: Decodable::decode(buf)?,
                gas_price: Decodable::decode(buf)?,
                gas_limit: Decodable::decode(buf)?,
                kind: Decodable::decode(buf)?,
                value: Decodable::decode(buf)?,
                encrypted_input: Decodable::decode(buf)?,
            },
        })
    }
}

impl SignableTransaction<Signature> for SeismicTransaction {
    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.tx.chain_id = chain_id;
    }

    fn encode_for_signing(&self, out: &mut dyn alloy_rlp::BufMut) {
        out.put_u8(self.tx.chain_id as u8);
        self.tx.encode(out)
    }

    fn payload_len_for_signature(&self) -> usize {
        1 + self.tx.length()
    }

    fn into_signed(self, signature: Signature) -> Signed<Self> {
        let mut buf = Vec::with_capacity(self.tx.encoded_len_with_signature(&signature, false));
        self.tx.encode_with_signature(&signature, &mut buf, false);
        let hash = keccak256(&buf);

        // Drop any v chain id value to ensure the signature format is correct at the time of
        // combination for an EIP-1559 transaction. V should indicate the y-parity of the
        // signature.
        Signed::new_unchecked(self, signature.with_parity_bool(), hash)
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
        &self.tx.encrypted_input
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
        0x64
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
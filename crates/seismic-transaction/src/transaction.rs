use alloy_consensus::{SignableTransaction, Signed, Transaction};
use alloy_eips::{eip2930::AccessList, eip7702::SignedAuthorization};
use alloy_primitives::{ChainId, Signature, TxKind, B256, U256};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::transaction_request::SeismicTransactionRequest;

/// Represents a seismic transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeismicTransaction {
    /// The base transaction data.
    #[serde(flatten)]
    pub tx: SeismicTransactionRequest,
}

impl SeismicTransaction {
    /// Seismic transaction type is 74
    pub const TRANSACTION_TYPE: u8 = 0x4A;

    pub(crate) fn decode_fields(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        Ok(Self { tx: SeismicTransactionRequest::decode_fields(buf)? })
    }
}

impl SignableTransaction<Signature> for SeismicTransaction {
    fn set_chain_id(&mut self, chain_id: ChainId) {
        self.tx.set_chain_id(chain_id);
    }

    fn encode_for_signing(&self, out: &mut dyn alloy_rlp::BufMut) {
        self.tx.encode_for_signing(out);
    }

    fn payload_len_for_signature(&self) -> usize {
        self.tx.payload_len_for_signature()
    }

    fn into_signed(self, signature: Signature) -> Signed<Self> {
        let signed_request = self.tx.into_signed(signature);
        let (tx_request, signature, hash) = signed_request.into_parts();
        let tx = SeismicTransaction { tx: tx_request };
        Signed::new_unchecked(tx, signature, hash)
    }
}

impl Transaction for SeismicTransaction {
    fn chain_id(&self) -> Option<ChainId> {
        self.tx.chain_id()
    }
    fn nonce(&self) -> u64 {
        self.tx.nonce()
    }
    fn gas_limit(&self) -> u128 {
        self.tx.gas_limit()
    }
    fn gas_price(&self) -> Option<u128> {
        self.tx.gas_price()
    }
    fn to(&self) -> TxKind {
        self.tx.to()
    }
    fn value(&self) -> U256 {
        self.tx.value()
    }
    fn input(&self) -> &[u8] {
        &self.tx.input()
    }
    fn max_fee_per_gas(&self) -> u128 {
        self.tx.max_fee_per_gas()
    }
    fn max_priority_fee_per_gas(&self) -> Option<u128> {
        self.tx.max_priority_fee_per_gas()
    }
    fn max_fee_per_blob_gas(&self) -> Option<u128> {
        self.tx.max_fee_per_blob_gas()
    }
    fn priority_fee_or_price(&self) -> u128 {
        self.tx.priority_fee_or_price()
    }
    fn ty(&self) -> u8 {
        self.tx.ty()
    }
    fn access_list(&self) -> Option<&AccessList> {
        self.tx.access_list()
    }
    fn blob_versioned_hashes(&self) -> Option<&[B256]> {
        self.tx.blob_versioned_hashes()
    }
    fn authorization_list(&self) -> Option<&[SignedAuthorization]> {
        self.tx.authorization_list()
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

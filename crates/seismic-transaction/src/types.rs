use crate::transaction_request::SeismicTransactionRequest;
use alloy_primitives::Bytes;
use alloy_serde::{OtherFields, WithOtherFields};
use reth_rpc_types::transaction::{
    EIP1559TransactionRequest, EIP2930TransactionRequest, EIP4844TransactionRequest,
    LegacyTransactionRequest, TransactionRequest
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Container type for various Seismic transaction requests.
///
/// Its variants correspond to specific allowed transactions:
/// 1. Legacy (pre-EIP2718) [`LegacyTransactionRequest`]
/// 2. EIP1559 [`EIP1559TransactionRequest`]
/// 3. EIP2930 (state access lists) [`EIP2930TransactionRequest`]
/// 4. EIP4844 [`EIP4844TransactionRequest`]
/// 5. Seismic [`SeismicTransactionRequest`]
#[derive(Debug)]
pub enum SeismicTypedTransactionRequest {
    /// Represents a Legacy (pre-EIP2718) transaction request.
    Legacy(LegacyTransactionRequest),
    /// Represents an EIP1559 transaction request.
    EIP1559(EIP1559TransactionRequest),
    /// Represents an EIP2930 (state access lists) transaction request.
    EIP2930(EIP2930TransactionRequest),
    /// Represents an EIP4844 transaction request.
    EIP4844(EIP4844TransactionRequest),
    /// Represents a Seismic transaction request.
    Seismic(SeismicTransactionRequest),
}
/// Seismic specific transaction field(s)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeismicTransactionFields {
    /// The secret data for the transaction
    #[serde(rename = "seismicInput")]
    pub seismic_input: Bytes,
}
impl From<SeismicTransactionFields> for OtherFields {
    fn from(value: SeismicTransactionFields) -> Self {
        serde_json::to_value(value).unwrap().try_into().unwrap()
    }
}

/// Either a normal ETH call or a signed/serialized one
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SeismicCallRequest {
    /// signed call request
    Bytes(Bytes),
    /// normal call request
    TransactionRequest(WithOtherFields<SeismicTransactionRequest>),
}

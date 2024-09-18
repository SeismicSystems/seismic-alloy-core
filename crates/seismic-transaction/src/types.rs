use crate::transaction::SeismicTransactionRequest;
use alloy_primitives::Bytes;
use alloy_serde::OtherFields;
use reth_rpc_types::transaction::{
    EIP1559TransactionRequest, EIP2930TransactionRequest, EIP4844TransactionRequest,
    LegacyTransactionRequest,
};
use seismic_types::preimage::value::PreImageValue;
use serde::{Deserialize, Serialize};

/// Represents secret data associated with a transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretData {
    /// The index of the secret data.
    pub index: usize,
    /// The preimage value of the secret data.
    pub preimage: PreImageValue,
    /// The type of the preimage.
    pub preimage_type: String,
    /// The salt value associated with the secret data.
    pub salt: Bytes,
}

/// Represents the fields of a seismic transaction.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeismicTransactionFields {
    /// A vector containing secret data associated with the transaction.
    /// This field is optional and will be skipped during serialization if it is `None`.
    #[serde(rename = "secretData", skip_serializing_if = "Option::is_none")]
    pub secret_data: Option<Vec<SecretData>>,
}

impl From<SeismicTransactionFields> for OtherFields {
    fn from(value: SeismicTransactionFields) -> Self {
        serde_json::to_value(value).unwrap().try_into().unwrap()
    }
}

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

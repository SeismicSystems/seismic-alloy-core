use alloy_primitives::Bytes;
use alloy_serde::OtherFields;
use seismic_preimages::PreImageValue;
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

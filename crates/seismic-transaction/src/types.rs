use alloy_primitives::Bytes;
use alloy_serde::OtherFields;
use seismic_preimages::PreImageValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretData {
    pub index: usize,
    pub preimage: PreImageValue,
    pub preimage_type: String,
    pub salt: Bytes,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeismicTransactionFields {
    #[serde(rename = "secretData", skip_serializing_if = "Option::is_none")]
    pub secret_data: Option<Vec<SecretData>>,
}

impl From<SeismicTransactionFields> for OtherFields {
    fn from(value: SeismicTransactionFields) -> Self {
        serde_json::to_value(value).unwrap().try_into().unwrap()
    }
}

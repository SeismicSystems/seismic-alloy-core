use crate::{
    seismic_util::{decrypt, Encryptable},
    transaction::SeismicTransactionRequest,
};
use alloy_primitives::{Bytes, TxKind, U256};
use alloy_rlp::{bytes, Decodable, Encodable};
use reth_rpc_types::{
    transaction::{
        EIP1559TransactionRequest, EIP2930TransactionRequest, EIP4844TransactionRequest,
        LegacyTransactionRequest,
    },
    OtherFields,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, hash::Hash, mem};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct PlainValue<T> {
    pub should_encrypt: bool,
    pub value: T,
}

impl<T: Encryptable + Debug + Clone + PartialEq + Eq + Serialize + for<'de> Deserialize<'de>>
    Encodable for PlainValue<T>
{
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        self.value.encode(out);
        self.should_encrypt.encode(out);
    }

    fn length(&self) -> usize {
        self.value.length() + self.should_encrypt.length()
    }
}

impl<T: Encryptable + Debug + Clone + PartialEq + Eq + Serialize + for<'de> Deserialize<'de>>
    Decodable for PlainValue<T>
{
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let value = T::decode(buf)?;
        let should_encrypt = bool::decode(buf)?;
        Ok(PlainValue { value, should_encrypt })
    }
}

impl<T: Encryptable + Debug + Clone + PartialEq + Eq + Serialize + for<'de> Deserialize<'de>>
    PlainValue<T>
{
    #[inline]
    pub fn size(&self) -> usize {
        mem::size_of::<T>() + mem::size_of::<bool>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct TxSeismicElement<T> {
    /// decrypted value is served as a cache for fast access of the encrypted value
    pub plainvalue: Option<PlainValue<T>>,
    /// encrypted value is assumed to not be changed during the lifetime of the struct
    /// this is already encoded for communication
    pub ciphertext: Vec<u8>, // maybe option instead of 0 ciphertext for plainvalue?
}

impl<T: Encryptable + Debug + Clone + PartialEq + Eq + Serialize + for<'de> Deserialize<'de>>
    TxSeismicElement<T>
{
    pub fn decrypt(&mut self, ciphertext: &Vec<u8>, nonce: u64) -> Result<(), alloy_rlp::Error> {
        if self.plainvalue.is_none() {
            let fresh_plainvalue = decrypt::<PlainValue<T>>(ciphertext, nonce)?;
            self.plainvalue = Some(fresh_plainvalue);
        }
        Ok(())
    }
}

impl<T: Encryptable + Debug + Clone + PartialEq + Eq + Serialize + for<'de> Deserialize<'de>>
    Encodable for TxSeismicElement<T>
{
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        // cypher text is assumed to not be changed during the lifetime of the struct
        // encryption is already encoded
        if let Some(plainvalue) = &self.plainvalue {
            if !plainvalue.should_encrypt {
                plainvalue.value.encode(out);
            } else {
                out.put_slice(&self.ciphertext);
            }
        } else {
            out.put_slice(&self.ciphertext);
        }
    }

    fn length(&self) -> usize {
        self.ciphertext.length()
    }
}

impl<T: Encryptable + Debug + Clone + PartialEq + Eq + Serialize + for<'de> Deserialize<'de>>
    Decodable for TxSeismicElement<T>
{
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let ciphertext = buf.to_vec();
        Ok(TxSeismicElement { plainvalue: None, ciphertext })
    }
}

pub type SeismicInput<
    T: Encryptable + Debug + Clone + PartialEq + Eq + Serialize + for<'de> Deserialize<'de> + Debug,
> = TxSeismicElement<T>;

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

// Seismic specific transaction field(s)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeismicTransactionFields {
    /// The secret data for the transaction
    #[serde(rename = "seismicInput")]
    pub seismic_input: Bytes, // ---> decryption ----> Bytes ----> EVM
}

impl From<SeismicTransactionFields> for OtherFields {
    fn from(value: SeismicTransactionFields) -> Self {
        serde_json::to_value(value).unwrap().try_into().unwrap()
    }
}

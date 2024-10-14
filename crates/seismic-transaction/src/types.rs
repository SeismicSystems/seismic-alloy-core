use reth_rpc_types::transaction::{LegacyTransactionRequest, EIP1559TransactionRequest, EIP2930TransactionRequest, EIP4844TransactionRequest};
use alloy_primitives::{U256, TxKind};
use serde::{Serialize, Deserialize};
use crate::seismic_util::decrypt;
use std::mem;
use std::{fmt::Debug, hash::Hash};
use alloy_rlp::{Decodable, Encodable, bytes};
use crate::seismic_util::Encryptable; 


pub struct PlainValue<T: Encryptable + Debug + Clone + PartialEq + Eq> {
    pub should_encrypt: bool,
    pub value: T,
}

impl<T: Encryptable + Debug + Clone + PartialEq + Eq> Encodable for PlainValue<T> {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        self.value.encode(out);
        self.should_encrypt.encode(out);
    }

    fn length(&self) -> usize {
        self.value.length() + self.should_encrypt.length()
    }
}

impl<T: Encryptable + Debug + Clone + PartialEq + Eq> Decodable for PlainValue<T> {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let value = T::decode(buf)?;
        let should_encrypt = bool::decode(buf)?;
        Ok(PlainValue {
            value,
            should_encrypt,
        })
    }
}

impl<T: Encryptable + Debug + Clone + PartialEq + Eq> PlainValue<T> {
    #[inline]
    pub fn size(&self) -> usize {
        mem::size_of::<T>() + mem::size_of::<bool>()
    }
    }
pub struct TxSeismicElement<T: Encryptable+Debug+Clone+PartialEq+Eq> {
    /// decrypted value is served as a cache for fast access of the encrypted value
    pub plainvalue: Option<PlainValue<T>>,
    /// encrypted value is assumed to not be changed during the lifetime of the struct
    /// this is already encoded for communication
    pub cyphertext: Vec<u8>, 
}
 
impl<T: Encryptable+Debug+Clone+PartialEq+Eq> TxSeismicElement<T> {
    pub fn decrypt(&mut self, ciphertext: &Vec<u8>, nonce: u64) -> Result<(), alloy_rlp::Error> {
        if self.plainvalue.is_none() {
            let fresh_plainvalue = decrypt::<PlainValue<T>>(ciphertext, nonce)?;
            self.plainvalue = Some(fresh_plainvalue);
        }
        Ok(())
    }
}

impl<T: Encryptable+Debug+Clone+PartialEq+Eq> Encodable for TxSeismicElement<T> {
    fn encode(&self, out: &mut dyn bytes::BufMut) {
        // cypher text is assumed to not be changed during the lifetime of the struct
        // encryption is already encoded
        if let Some(plainvalue) = &self.plainvalue {
            if !plainvalue.should_encrypt {
                plainvalue.value.encode(out);
            } else {
                out.put_slice(&self.cyphertext);
            }
        } else {
            out.put_slice(&self.cyphertext);
        }
    }

    fn length(&self) -> usize {
        self.cyphertext.length()
    }
}

impl<T: Encryptable+Debug+Clone+PartialEq+Eq> Decodable for TxSeismicElement<T> {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let cyphertext = buf.to_vec();
        Ok(TxSeismicElement {
            plainvalue: None,
            cyphertext,
        })
    }
}

type SeismicInput<T: Encryptable+Debug+Clone+PartialEq+Eq> = TxSeismicElement<T>;


/// Represents an Seismic transaction request
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeismicTransactionRequest<T: Encryptable+Debug+Clone+PartialEq+Eq> 
where TxSeismicElement<T>: Encryptable + Debug + Clone + PartialEq + Eq {
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
    pub input: SeismicInput<T>,
    /// The optional chain ID for the transaction
    pub chain_id: u64,
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
pub enum SeismicTypedTransactionRequest<T: Encryptable + Debug + Clone + PartialEq + Eq> {
    /// Represents a Legacy (pre-EIP2718) transaction request.
    Legacy(LegacyTransactionRequest),
    /// Represents an EIP1559 transaction request.
    EIP1559(EIP1559TransactionRequest),
    /// Represents an EIP2930 (state access lists) transaction request.
    EIP2930(EIP2930TransactionRequest),
    /// Represents an EIP4844 transaction request.
    EIP4844(EIP4844TransactionRequest),
    /// Represents a Seismic transaction request.
    Seismic(SeismicTransactionRequest<T>),
}

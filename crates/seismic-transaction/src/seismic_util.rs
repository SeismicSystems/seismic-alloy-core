use alloy_primitives::{keccak256, Bytes, ChainId, Signature, TxKind, B256, U256};
use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, OsRng as AesRng},
    Aes256Gcm, Key,
};
use alloy_rlp::{Decodable, Encodable, Error};
use once_cell::sync::Lazy;
use paste::paste;

// #[cfg(any(test, feature = "reth-codec"))]
// use reth_codecs::Compact;

#[cfg(not(feature = "std"))]
// use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

// Static variable that will hold the generated key, initialized lazily
static AES_KEY: Lazy<Key<Aes256Gcm>> = Lazy::new(|| {
    let rng = AesRng::default();
    let key: Key<Aes256Gcm> = Aes256Gcm::generate_key(rng);
    return key;
});

fn nonce_to_generic_array(nonce: u64) -> GenericArray<u8, <Aes256Gcm as AeadCore>::NonceSize> {
    let mut nonce_bytes = nonce.to_be_bytes().to_vec();
    let crypto_nonce_size = GenericArray::<u8, <Aes256Gcm as AeadCore>::NonceSize>::default().len();
    nonce_bytes.resize(crypto_nonce_size, 0); // pad for crypto
    GenericArray::clone_from_slice(&nonce_bytes)
}

pub trait Encryptable: Encodable + Decodable {}
impl<T: Encodable + Decodable> Encryptable for T {}

pub fn decrypt<T>(ciphertext: &Vec<u8>, nonce: u64) -> alloy_rlp::Result<T>
where
    T: Encryptable,
{
    let cipher = Aes256Gcm::new(&AES_KEY);
    let nonce = nonce_to_generic_array(nonce);
    let buf = cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|_err| Error::Custom("Failed to decrypt seismic transaction"))?;
    T::decode(&mut &buf[..])
}

pub fn encrypt<T: Encryptable>(plaintext: &T, nonce: u64) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new(&AES_KEY);
    let nonce = nonce_to_generic_array(nonce);
    let mut buf = Vec::new();
    plaintext.encode(&mut buf);
    // Returns an error if the buffer has insufficient capacity to store the
    // resulting ciphertext message.
    cipher
        .encrypt(&nonce, buf.as_ref())
        .map_err(|_err| Error::Custom("Failed to encrypt seismic transaction"))
}
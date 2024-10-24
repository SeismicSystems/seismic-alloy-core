use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit},
    Aes256Gcm, Key,
};
use alloy_rlp::{Decodable, Encodable, Error};
use once_cell::sync::Lazy;

/// The AES-256-GCM key used for encryption and decryption. 
/// Hardcoded to be 0 for testing purposes for now 
/// (to be used in tandem with seismic-viem)
static AES_KEY: Lazy<Key<Aes256Gcm>> = Lazy::new(|| {
    // Define a fixed byte array for the key
    let key_bytes: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    // Create the key from the fixed byte array
    Key::<Aes256Gcm>::from_slice(&key_bytes).clone()
});

/// Converts a 64-bit nonce to a generic array of bytes.
fn nonce_to_generic_array(nonce: u64) -> GenericArray<u8, <Aes256Gcm as AeadCore>::NonceSize> {
    let mut nonce_bytes = nonce.to_be_bytes().to_vec();
    let crypto_nonce_size = GenericArray::<u8, <Aes256Gcm as AeadCore>::NonceSize>::default().len();
    println!("crypto_nonce_size: {}", crypto_nonce_size);
    nonce_bytes.resize(crypto_nonce_size, 0); // pad for crypto
    GenericArray::clone_from_slice(&nonce_bytes)
}

/// Trait for types that can be encrypted and decrypted
pub trait Encryptable: Encodable + Decodable {}
impl<T: Encodable + Decodable> Encryptable for T {}

/// Decrypts the given ciphertext using the AES-256-GCM algorithm.
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

/// Encrypts the given plaintext using the AES-256-GCM algorithm.
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Bytes, hex::FromHex};

    #[test]
    fn test_seismic_encrypt_decrypt() {
        // Input data: selector + value (10 in hex)
        let input = Bytes::from_hex("0x3fb5c1cb000000000000000000000000000000000000000000000000000000000000000a").unwrap();
        let nonce = 1;

        // Encrypt the input
        let encrypted = encrypt(&input, nonce).expect("Encryption failed");

        // Decrypt the encrypted data
        let decrypted = decrypt::<Bytes>(&encrypted, nonce).expect("Decryption failed");

        // Assert that the decrypted data matches the original input
        assert_eq!(input, decrypted, "Decrypted data does not match original input");
    }
}

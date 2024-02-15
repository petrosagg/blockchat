//! The definition of all cryptographic primitives used in BlockChat.

use rsa::sha2::Sha256;
use serde::{Deserialize, Serialize};
use rsa::{RsaPrivateKey, RsaPublicKey};
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::signature::{Signer, Verifier};
use rsa::signature::SignatureEncoding;


pub const KEY_SIZE: usize = 2048;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Hash;

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PublicKey(RsaPublicKey);

impl PublicKey {
    pub fn verify<T: Serialize>(&self, signature: Signed<T>) -> Result<(), rsa::signature::Error> {
        let verifying_key = VerifyingKey::<Sha256>::new(self.0.clone());
        let data_encoded = serde_json::to_vec(&(signature.data)).unwrap();
        let helper: &[u8] = &signature.signature;
        let signature_decoded = Signature::try_from(helper).unwrap();
        verifying_key.verify(&data_encoded, &signature_decoded)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PrivateKey(RsaPrivateKey);

impl PrivateKey {
    pub fn sign<T: Serialize>(&self, data: T) -> Signed<T> {
        let signing_key = SigningKey::<Sha256>::new(self.0.clone());
        let data_encoded = serde_json::to_vec(&data).unwrap();
        Signed{signature: signing_key.sign(&data_encoded).to_vec(), data}
    }
}

pub fn generate_keypair() -> (PrivateKey, PublicKey) {
    let mut rng = rand::thread_rng();

    let private_key = RsaPrivateKey::new(&mut rng, KEY_SIZE).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    (PrivateKey(private_key), PublicKey(public_key))
}


/// A container of signed data
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Signed<T> {
    /// A signature proving that the sender wallet created this transaction.
    pub signature: Vec<u8>,
    /// The data being signed,
    pub data: T,
}

impl<T: Serialize> Signed<T> {
    pub fn new(data: T, _key: &PrivateKey) -> Self {
        // TODO: actually sign this
        Self {
            signature: Signature,
            data,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn generate_keypair_test() {
        let (private_key, public_key) = generate_keypair();

        assert!(private_key.0.validate().is_ok());
        assert!(private_key.0.to_public_key() == public_key.0);
    }

    #[test]
    fn sign_verify_test() {
        let (private_key, public_key) = generate_keypair();
        let (_, other_public_key) = generate_keypair();
        let data = b"Hello World!";
        let signature = private_key.sign(data);

        assert!(public_key.verify(signature.clone()).is_ok());
        assert!(other_public_key.verify(signature).is_err());
    }
}

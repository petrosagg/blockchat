//! The definition of all cryptographic primitives used in BlockChat.

use std::cmp::Ordering;
use std::fmt;

use base64::{display::Base64Display, engine::general_purpose::STANDARD_NO_PAD};
use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::sha2::{Digest, Sha256};
use rsa::signature::SignatureEncoding;
use rsa::signature::{Signer, Verifier};
use rsa::{RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub const KEY_SIZE: usize = 2048;

#[derive(Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn digest<T: Serialize>(data: T) -> Self {
        let data_encoded = serde_json::to_vec(&data).unwrap();
        Self(Sha256::digest(data_encoded).into())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PublicKey {
    key: RsaPublicKey,
    hash: Hash,
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for PublicKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hash.cmp(&other.hash)
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Base64Display::new(&self.hash.0, &STANDARD_NO_PAD))
    }
}

impl PublicKey {
    /// Constructs an invalid public key which does not have a corresponding private key.
    pub fn invalid() -> Self {
        Self {
            key: RsaPublicKey::new_unchecked(0u64.into(), 0u64.into()),
            hash: Default::default(),
        }
    }

    pub fn verify<T: Serialize>(&self, signature: Signed<T>) -> Result<Signed<T>> {
        let verifying_key = VerifyingKey::<Sha256>::new(self.key.clone());
        let hash = Hash::digest(&signature.data);
        if hash != signature.hash {
            return Err(Error::InvalidSignature(Default::default()));
        }
        let signature_decoded = Signature::try_from(&*signature.signature).unwrap();
        verifying_key.verify(&hash.0, &signature_decoded)?;
        Ok(signature)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PrivateKey(RsaPrivateKey);

impl PrivateKey {
    pub fn sign<T: Serialize>(&self, data: T) -> Signed<T> {
        let signing_key = SigningKey::<Sha256>::new(self.0.clone());
        let hash = Hash::digest(&data);

        Signed {
            signature: signing_key.sign(&hash.0).to_vec(),
            hash,
            data,
        }
    }
}

pub fn generate_keypair() -> (PrivateKey, PublicKey) {
    let mut rng = rand::thread_rng();

    let private_key = RsaPrivateKey::new(&mut rng, KEY_SIZE).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    let hash = Hash::digest(&public_key);
    let public_key = PublicKey {
        key: public_key,
        hash,
    };

    (PrivateKey(private_key), public_key)
}

/// A container of signed data
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Signed<T> {
    /// A signature proving that the sender wallet created this transaction.
    pub signature: Vec<u8>,
    /// The hash of the data.
    pub hash: Hash,
    /// The data being signed,
    pub data: T,
}

impl<T: fmt::Debug> fmt::Debug for Signed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Signed")
            .field("signature", &"...")
            .field("hash", &self.hash)
            .field("data", &self.data)
            .finish()
    }
}

impl<T: Serialize + Clone> Signed<T> {
    /// Creates an invalid a signed object whose signature is invalid. This is used for generating
    /// the genesis block and for testing.
    pub fn new_invalid(data: T) -> Signed<T> {
        Signed {
            signature: vec![],
            hash: Hash::digest(data.clone()),
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
        assert!(private_key.0.to_public_key() == public_key.key);
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

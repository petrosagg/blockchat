//! The definition of all cryptographic primitives used in BlockChat.

use serde::{Deserialize, Serialize};
use rsa::{RsaPrivateKey, RsaPublicKey};


#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Hash;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PublicKey;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PrivateKey;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Signature;

pub fn generate_keypair() -> (RsaPrivateKey, RsaPublicKey) {
    // TODO: Actually generate these randomly
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

    (private_key, public_key)
}


/// A container of signed data
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Signed<T> {
    /// The hash of this transaction.
    pub hash: Hash,
    /// A signature proving that the sender wallet created this transaction.
    pub signature: Signature,
    /// The data being signed,
    pub data: T,
}

impl<T: Serialize> Signed<T> {
    pub fn new(data: T, _key: &PrivateKey) -> Self {
        // TODO: actually sign this
        Self {
            hash: Hash,
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

        assert!((private_key.validate()).is_ok());
        assert!(private_key.to_public_key() == public_key);
    }

}

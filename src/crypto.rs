//! The definition of all cryptographic primitives used in BlockChat.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Hash;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PublicKey;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PrivateKey;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Signature;

pub fn generate_keypair() -> (PublicKey, PrivateKey) {
    // TODO: Actually generate these randomly
    (PublicKey, PrivateKey)
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
    pub fn new(data: T, key: &PrivateKey) -> Self {
        // TODO: actually sign this
        Self {
            hash: Hash,
            signature: Signature,
            data,
        }
    }
}

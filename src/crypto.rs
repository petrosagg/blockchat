//! The definition of all cryptographic primitives used in BlockChat.

use std::fmt;
use std::str::FromStr;

use rsa::pkcs1v15::{Signature, SigningKey, VerifyingKey};
use rsa::sha2::{Digest, Sha256};
use rsa::signature::SignatureEncoding;
use rsa::signature::{Signer, Verifier};
use rsa::traits::PublicKeyParts;
use rsa::{BigUint, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_with::base64::Base64;
use serde_with::{serde_as, DeserializeFromStr, SerializeDisplay};

use crate::error::{Error, Result};

pub const KEY_SIZE: usize = 2048;

#[derive(
    Default, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, DeserializeFromStr, SerializeDisplay,
)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn digest<T: Serialize>(data: T) -> Self {
        let data_encoded = serde_json::to_vec(&data).unwrap();
        Self(Sha256::digest(data_encoded).into())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for Hash {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // TODO: error handling
        let hash = hex::decode(s).unwrap().try_into().unwrap();
        Ok(Hash(hash))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PublicKey {
    key: RsaPublicKey,
}

#[serde_as]
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
struct EncodedPublicKey {
    #[serde_as(as = "Base64")]
    modulus: Vec<u8>,
    #[serde_as(as = "Base64")]
    public_exponent: Vec<u8>,
}

impl Serialize for PublicKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let encoded = EncodedPublicKey {
            modulus: self.key.n().to_bytes_be(),
            public_exponent: self.key.e().to_bytes_be(),
        };
        encoded.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let encoded = EncodedPublicKey::deserialize(deserializer)?;
        let modulus = BigUint::from_bytes_be(&encoded.modulus);
        let public_exponent = BigUint::from_bytes_be(&encoded.public_exponent);
        let key = RsaPublicKey::new(modulus, public_exponent).unwrap();
        Ok(PublicKey { key })
    }
}

impl PublicKey {
    /// Constructs an invalid public key which does not have a corresponding private key.
    pub fn invalid() -> Self {
        Self {
            key: RsaPublicKey::new_unchecked(0u64.into(), 0u64.into()),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, SerializeDisplay, DeserializeFromStr)]
pub struct Address(Hash);

impl Address {
    pub fn from_public_key(key: &PublicKey) -> Self {
        Self(Hash::digest(key))
    }

    pub fn invalid() -> Self {
        Address(Hash::default())
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", base_62::base62::encode(&self.0 .0))
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl FromStr for Address {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // TODO: error handling
        let hash = base_62::base62::decode(s).unwrap().try_into().unwrap();
        Ok(Address(Hash(hash)))
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
            public_key: PublicKey {
                key: RsaPublicKey::from(&self.0),
            },
            hash,
            data,
        }
    }
}

pub fn generate_keypair() -> (PrivateKey, PublicKey) {
    let mut rng = rand::thread_rng();

    let private_key = RsaPrivateKey::new(&mut rng, KEY_SIZE).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);
    let public_key = PublicKey { key: public_key };

    (PrivateKey(private_key), public_key)
}

/// A container of signed data
#[serde_as]
#[derive(Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Signed<T> {
    // The public key used for the signature of the hash of the data.
    pub public_key: PublicKey,
    /// The signature of the hash of the data.
    #[serde_as(as = "Base64")]
    pub signature: Vec<u8>,
    /// The hash of the data.
    pub hash: Hash,
    /// The data.
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
            public_key: PublicKey::invalid(),
            signature: vec![],
            hash: Hash::digest(data.clone()),
            data,
        }
    }

    pub fn verify(&self) -> Result<()> {
        let verifying_key = VerifyingKey::<Sha256>::new(self.public_key.key.clone());
        let hash = Hash::digest(&self.data);
        if hash != self.hash {
            return Err(Error::InvalidSignature(Default::default()));
        }
        let signature_decoded = Signature::try_from(&*self.signature).unwrap();
        verifying_key.verify(&self.hash.0, &signature_decoded)?;
        Ok(())
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
        let (private_key, _) = generate_keypair();
        let data = b"Hello World!";
        let signature = private_key.sign(data);

        assert!(signature.verify().is_ok());
    }
}

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to validate signature")]
    InvalidSignature(#[from] rsa::signature::Error),
    #[error("the wallet does not have sufficient funds")]
    InsufficientFunds,
    #[error("expected nonce to be at least ${1} but was ${0}")]
    NonceReused(u64, u64),
    #[error("block signer is not the expected validator")]
    InvalidBlockValidator,
}

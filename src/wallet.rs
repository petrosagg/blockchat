use serde::{Deserialize, Serialize};

use crate::crypto::{self, PrivateKey, PublicKey, Signed};

pub struct Wallet {
    /// The public key of this wallet.
    pub public_key: PublicKey,
    /// The private key of this wallet.
    private_key: PrivateKey,
    /// An auto-increment nonce used to sign transactions.
    nonce: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Transaction {
    /// The public key of the sending wallet.
    sender_address: PublicKey,
    /// The public key of the receiving wallet.
    receiver_address: PublicKey,
    /// The kind of this transaction.
    kind: TransactionKind,
    /// The sender nonce.
    nonce: u64,
}

impl Wallet {
    pub fn new() -> Self {
        // let (public_key, private_key) = crypto::generate_keypair();
        // Self {
        //     public_key,
        //     private_key,
        //     nonce: 0,
        // }
        todo!();
    }

    pub fn sign_coin_transaction(
        &mut self,
        receiver: &PublicKey,
        amount: u64,
    ) -> Signed<Transaction> {
        let nonce = self.nonce;
        self.nonce += 1;

        let tx = Transaction {
            sender_address: self.public_key.clone(),
            receiver_address: receiver.clone(),
            kind: TransactionKind::Coin(amount),
            nonce,
        };

        Signed::new(tx, &self.private_key)
    }

    pub fn create_message_transaction(&self /* args */) -> Transaction {
        todo!()
    }

    pub fn sign_transaction(&self, _tx: Transaction) -> Signed<Transaction> {
        // 1. Calculate the hash of the transaction
        // 2. Sign it using the provided key
        todo!()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TransactionKind {
    /// A coin transaction transferring the specified amount.
    Coin(u64),
    /// A message transaction transferring the specified message.
    Message(String),
    // A staking transaction locking up the specified amount.
    Stake(u64),
}

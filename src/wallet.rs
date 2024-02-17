use serde::{Deserialize, Serialize};

use crate::crypto::{self, PrivateKey, PublicKey, Signed};

pub struct Wallet {
    /// The private key of this wallet.
    private_key: PrivateKey,
    /// The public key of this wallet.
    pub public_key: PublicKey,
    /// An auto-increment nonce used to sign transactions.
    nonce: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Transaction {
    /// The public key of the sending wallet.
    pub sender_address: PublicKey,
    /// The kind of this transaction.
    pub kind: TransactionKind,
    /// The sender nonce.
    pub nonce: u64,
}

impl Wallet {
    pub fn new() -> Self {
        let (private_key, public_key) = crypto::generate_keypair();
        Self {
            private_key,
            public_key,
            nonce: 0,
        }
    }

    fn create_transaction(&mut self,
        kind: TransactionKind
    ) -> Transaction {
        let nonce = self.nonce;
        self.nonce += 1;

        Transaction {
            sender_address: self.public_key.clone(),
            kind,
            nonce
        }
    }

    pub fn create_coin_transaction(
        &mut self,
        receiver: PublicKey,
        amount: u64,
    ) -> Signed<Transaction> {
        let tx = self.create_transaction(TransactionKind::Coin(amount, receiver));

        self.private_key.sign(tx)
    }

    pub fn create_message_transaction(
        &mut self,
        receiver: PublicKey,
        message: String,
    ) -> Signed<Transaction> {
        let tx = self.create_transaction(TransactionKind::Message(message, receiver));

        self.private_key.sign(tx)
    }

    pub fn create_stake_transaction(
        &mut self,
        amount: u64,
    ) -> Signed<Transaction> {
        let tx = self.create_transaction(TransactionKind::Stake(amount));

        self.private_key.sign(tx)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionKind {
    /// A coin transaction transferring the specified amount to the receiver.
    Coin(u64, PublicKey),
    /// A message transaction transferring the specified message to the receiver.
    Message(String, PublicKey),
    // A staking transaction locking up the specified amount.
    Stake(u64),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_transactions_test() {
        let mut wallet = Wallet::new();

        let sender = wallet.public_key.clone();
        let (_, receiver) = crate::crypto::generate_keypair();
        let coin_amount = 40;
        let stake_amount = 100;
        let message = String::from("Hello World!");

        let mut tx = wallet.create_coin_transaction(receiver.clone(), coin_amount);

        assert!(sender.verify(tx.clone()).is_ok());
        assert!(tx.data.sender_address == wallet.public_key);
        assert!(tx.data.kind == TransactionKind::Coin(coin_amount, receiver.clone()));
        assert!(tx.data.nonce == 0);

        tx = wallet.create_message_transaction(receiver.clone(), message.clone());

        assert!(sender.verify(tx.clone()).is_ok());
        assert!(tx.data.sender_address == wallet.public_key);
        assert!(tx.data.kind == TransactionKind::Message(message, receiver));
        assert!(tx.data.nonce == 1);

        tx = wallet.create_stake_transaction(stake_amount.clone());

        assert!(sender.verify(tx.clone()).is_ok());
        assert!(tx.data.sender_address == wallet.public_key);
        assert!(tx.data.kind == TransactionKind::Stake(stake_amount));
        assert!(tx.data.nonce == 2);
    }
}

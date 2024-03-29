use serde::{Deserialize, Serialize};

use crate::crypto::{Address, PublicKey, Signed};
use crate::error::{Error, Result};

const FEE_PERCENT: u64 = 3;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wallet {
    /// The address of this wallet.
    pub address: Address,
    /// The current BCC balance of the wallet.
    pub balance: u64,
    /// The currently staked amount.
    pub stake: u64,
    /// An auto-increment nonce used to sign transactions.
    pub nonce: u64,
}

impl Wallet {
    pub fn from_address(address: Address) -> Self {
        Self {
            address,
            balance: 0,
            stake: 0,
            nonce: 0,
        }
    }

    pub fn from_public_key(public_key: &PublicKey) -> Self {
        Self::from_address(Address::from_public_key(public_key))
    }

    /// The amount of BCC available to use for transactions.
    pub fn available_funds(&self) -> u64 {
        self.balance - self.stake
    }

    /// The amount of BCC staked.
    pub fn staked_amount(&self) -> u64 {
        self.stake
    }

    fn create_tx(&self, kind: TransactionKind) -> Transaction {
        Transaction {
            sender_address: self.address.clone(),
            kind,
            nonce: self.nonce,
        }
    }

    /// Validates the provided transaction given the current wallet's state.
    pub fn validate_tx(&mut self, tx: Signed<Transaction>) -> Result<Signed<Transaction>> {
        tx.verify()?;
        // If this is our transaction we must also verify that we have sufficient funds.
        if tx.data.sender_address == self.address {
            if tx.data.nonce < self.nonce {
                return Err(Error::NonceReused(tx.data.nonce, self.nonce));
            }
            let fees = tx.data.fees();
            match &tx.data.kind {
                TransactionKind::Coin(amount, _) => {
                    if amount + fees > self.available_funds() {
                        return Err(Error::InsufficientFunds);
                    }
                }
                TransactionKind::Message(_, _) => {
                    if fees > self.available_funds() {
                        return Err(Error::InsufficientFunds);
                    }
                }
                TransactionKind::Stake(amount) => {
                    // Can only stake up to the current balance
                    if *amount > self.balance {
                        return Err(Error::InsufficientFunds);
                    }
                }
            }
        }
        Ok(tx)
    }

    /// Applies the provided transaction, provided it's valid
    /// transaction is valid. Returns an error if the transaction is invalid.
    pub fn apply_tx(&mut self, tx: Signed<Transaction>) -> Result<()> {
        let tx = self.validate_tx(tx)?.data;
        // If this is our transaction we must subtract the money moved and fees from our balance.
        if tx.sender_address == self.address {
            self.nonce = tx.nonce + 1;
            self.balance -= tx.fees();
            match tx.kind {
                TransactionKind::Coin(amount, _) => self.balance -= amount,
                TransactionKind::Message(_, _) => {}
                TransactionKind::Stake(amount) => self.stake = amount,
            }
        }
        // Finally, if this transaction moves money into this wallet we must add it to our balance.
        if let TransactionKind::Coin(amount, receiver) = tx.kind {
            if receiver == self.address {
                self.balance += amount;
            }
        }
        Ok(())
    }

    pub fn create_coin_tx(&self, receiver: Address, amount: u64) -> Transaction {
        self.create_tx(TransactionKind::Coin(amount, receiver))
    }

    pub fn create_message_tx(&self, receiver: Address, message: String) -> Transaction {
        self.create_tx(TransactionKind::Message(message, receiver))
    }

    pub fn create_stake_tx(&self, amount: u64) -> Transaction {
        self.create_tx(TransactionKind::Stake(amount))
    }

    pub fn add_funds(&mut self, amount: u64) {
        self.balance += amount;
    }

    pub fn set_stake(&mut self, amount: u64) {
        assert!(amount <= self.balance);
        self.stake = amount;
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Transaction {
    /// The public key of the sending wallet.
    pub sender_address: Address,
    /// The kind of this transaction.
    pub kind: TransactionKind,
    /// The alice_key nonce.
    pub nonce: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum TransactionKind {
    /// A coin transaction transferring the specified amount to the receiver.
    Coin(u64, Address),
    /// A message transaction transferring the specified message to the receiver.
    Message(String, Address),
    // A staking transaction locking up the specified amount.
    Stake(u64),
}

impl Transaction {
    /// Calculates the cost of this transaction.
    pub fn cost(&self) -> u64 {
        let value = match &self.kind {
            TransactionKind::Coin(amount, _) => *amount,
            TransactionKind::Message(_, _) => 0,
            TransactionKind::Stake(_) => 0,
        };
        self.fees() + value
    }

    /// Calculates the required fees of this transaction.
    pub fn fees(&self) -> u64 {
        match &self.kind {
            // TODO: should we charge a minimum amount when the calculation rounds down to zero?
            TransactionKind::Coin(amount, _) => (amount * FEE_PERCENT) / 100,
            TransactionKind::Message(msg, _) => msg.len() as u64,
            TransactionKind::Stake(_) => 0,
        }
    }

    pub fn receiver(&self) -> Option<Address> {
        match &self.kind {
            TransactionKind::Coin(_, receiver) | TransactionKind::Message(_, receiver) => {
                Some(receiver.clone())
            }
            TransactionKind::Stake(_) => None,
        }
    }
}

#[cfg(test)]
pub mod test {
    use crate::crypto::PrivateKey;

    use super::*;

    /// Creates a test wallet with 1M BCC as initial funds
    pub fn setup_test_wallet(initial_balance: u64) -> (Wallet, PublicKey, PrivateKey) {
        let (wallet_key, wallet_public_key) = crate::crypto::generate_keypair();
        let mut wallet = Wallet::from_public_key(&wallet_public_key);
        // Create Alice's keypair and give some initial funds to the test wallet
        let (funder_key, funder_public_key) = crate::crypto::generate_keypair();
        let initial_funds = Transaction {
            sender_address: Address::from_public_key(&funder_public_key),
            kind: TransactionKind::Coin(initial_balance, wallet.address.clone()),
            nonce: 0,
        };
        wallet.apply_tx(funder_key.sign(initial_funds)).unwrap();
        (wallet, wallet_public_key, wallet_key)
    }

    pub fn setup_default_test_wallet() -> (Wallet, PublicKey, PrivateKey) {
        setup_test_wallet(1_000_000)
    }

    #[test]
    fn test_coin_transaction() {
        let (mut sender_wallet, _, sender_key) = setup_default_test_wallet();
        let (mut receiver_wallet, _, _receiver_key) = setup_default_test_wallet();

        let coin_amount = 100;
        let tx = sender_wallet.create_coin_tx(receiver_wallet.address.clone(), coin_amount);
        let signed_tx = sender_key.sign(tx.clone());

        // First validate that the tx is well formed
        assert_eq!(
            tx,
            Transaction {
                sender_address: sender_wallet.address.clone(),
                kind: TransactionKind::Coin(coin_amount, receiver_wallet.address.clone()),
                nonce: 0,
            }
        );
        assert_eq!(tx.fees(), 3);

        // Apply the transaction to the sender wallet and verify funds adjust correctly.
        sender_wallet.apply_tx(signed_tx.clone()).unwrap();
        assert_eq!(sender_wallet.available_funds(), 1_000_000 - 100 - 3);
        assert_eq!(sender_wallet.nonce, 1);

        // Apply the transaction to the receiver wallet and verify funds adjust correctly.
        receiver_wallet.apply_tx(signed_tx.clone()).unwrap();
        assert_eq!(receiver_wallet.available_funds(), 1_000_000 + 100);
        assert_eq!(receiver_wallet.nonce, 0);
    }

    #[test]
    fn test_message_transaction() {
        let (mut sender_wallet, _, sender_key) = setup_default_test_wallet();
        let (mut receiver_wallet, _, _receiver_key) = setup_default_test_wallet();

        let message = String::from("Hello World!");
        let expected_fees = message.len() as u64;
        let tx = sender_wallet.create_message_tx(receiver_wallet.address.clone(), message.clone());
        let signed_tx = sender_key.sign(tx.clone());

        // First validate that the tx is well formed
        assert_eq!(
            tx,
            Transaction {
                sender_address: sender_wallet.address.clone(),
                kind: TransactionKind::Message(message, receiver_wallet.address.clone()),
                nonce: 0,
            }
        );
        assert_eq!(tx.fees(), expected_fees);

        // Apply the transaction to the sender wallet and verify funds adjust correctly.
        sender_wallet.apply_tx(signed_tx.clone()).unwrap();
        assert_eq!(sender_wallet.available_funds(), 1_000_000 - expected_fees);
        assert_eq!(sender_wallet.nonce, 1);

        // Apply the transaction to the receiver wallet and verify funds adjust correctly.
        receiver_wallet.apply_tx(signed_tx.clone()).unwrap();
        assert_eq!(receiver_wallet.available_funds(), 1_000_000);
        assert_eq!(receiver_wallet.nonce, 0);
    }

    #[test]
    fn test_stake_transaction() {
        let (mut sender_wallet, _, sender_key) = setup_default_test_wallet();

        let stake_amount = 100;
        let tx = sender_wallet.create_stake_tx(stake_amount);
        let signed_tx = sender_key.sign(tx.clone());

        // First validate that the tx is well formed
        assert_eq!(
            tx,
            Transaction {
                sender_address: sender_wallet.address.clone(),
                kind: TransactionKind::Stake(stake_amount),
                nonce: 0,
            }
        );
        assert_eq!(tx.fees(), 0);

        // Apply the transaction to the sender wallet and verify funds adjust correctly.
        sender_wallet.apply_tx(signed_tx.clone()).unwrap();
        assert_eq!(sender_wallet.available_funds(), 1_000_000 - stake_amount);
        assert_eq!(sender_wallet.stake, stake_amount);
        assert_eq!(sender_wallet.nonce, 1);
    }

    #[test]
    fn test_coin_insufficient_funds() {
        let (mut sender_wallet, _, sender_key) = setup_default_test_wallet();
        let (receiver_wallet, _, _receiver_key) = setup_default_test_wallet();

        // Beware of ceil.
        let coin_amount = 970_875;
        let tx = sender_wallet.create_coin_tx(receiver_wallet.address.clone(), coin_amount);
        let signed_tx = sender_key.sign(tx.clone());

        let result = sender_wallet.apply_tx(signed_tx.clone());
        assert!(matches!(result, Err(Error::InsufficientFunds)));
        assert_eq!(sender_wallet.nonce, 0);
    }

    #[test]
    fn test_message_insufficient_funds() {
        let (mut sender_wallet, _, sender_key) = setup_test_wallet(23);
        let (receiver_wallet, _, _receiver_key) = setup_default_test_wallet();

        let message = String::from("These are 24 characters.");
        let tx = sender_wallet.create_message_tx(receiver_wallet.address.clone(), message);
        let signed_tx = sender_key.sign(tx.clone());

        let result = sender_wallet.apply_tx(signed_tx.clone());
        assert!(matches!(result, Err(Error::InsufficientFunds)));
        assert_eq!(sender_wallet.nonce, 0);
    }

    #[test]
    fn test_stake_insufficient_funds() {
        let (mut sender_wallet, _, sender_key) = setup_default_test_wallet();

        let stake_amount = 1_000_001;
        let tx = sender_wallet.create_stake_tx(stake_amount);
        let signed_tx = sender_key.sign(tx.clone());

        let result = sender_wallet.apply_tx(signed_tx.clone());
        assert!(matches!(result, Err(Error::InsufficientFunds)));
        assert_eq!(sender_wallet.nonce, 0);
    }
}

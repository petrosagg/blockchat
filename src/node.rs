use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::crypto::{Hash, PrivateKey, PublicKey, Signed};
use crate::error::{Error, Result};
use crate::network::Network;
use crate::wallet::{Transaction, TransactionKind, Wallet};

const MINT_INTERVAL: Duration = Duration::from_secs(5);

pub struct Node {
    /// The maximum number of transactions contained in each block.
    capacity: usize,
    /// The set of signed but not necessarily valid transactions waiting to be included in a block.
    pending_transactions: BTreeMap<(PublicKey, u64), Signed<Transaction>>,
    /// The current blockchain.
    blockchain: Vec<Signed<Block>>,
    /// The public key of the wallet of this node.
    public_key: PublicKey,
    /// The private key of the wallet of this node.
    private_key: PrivateKey,
    /// The state of each known wallet indexed by public key. We use a BTreeMap to always maintain
    /// the wallets in sorted public key order which helps perform the validator election.
    wallets: BTreeMap<PublicKey, Wallet>,
    /// This node's handle to the network
    network: Box<dyn Network<Message>>,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("capacity", &self.capacity)
            .field("pending_transactions", &self.pending_transactions)
            .field("blockchain", &self.blockchain)
            .field("public_key", &self.public_key)
            .field("private_key", &"REDACTED")
            .field("wallets", &self.wallets)
            .finish()
    }
}

impl Node {
    pub fn new(
        public_key: PublicKey,
        private_key: PrivateKey,
        genesis_validator: PublicKey,
        genesis_funds: u64,
        capacity: usize,
        network: impl Network<Message> + 'static,
    ) -> Self {
        let genesis_tx = Transaction {
            sender_address: PublicKey::invalid(),
            kind: TransactionKind::Coin(genesis_funds, genesis_validator.clone()),
            nonce: 0,
        };

        let genesis_block = Block {
            timestamp: DateTime::<Utc>::MIN_UTC,
            transactions: vec![Signed::new_invalid(genesis_tx)],
            validator: PublicKey::invalid(),
            parent_hash: Hash::default(),
        };

        let mut wallets = BTreeMap::new();
        let mut genesis_wallet = Wallet::with_public_key(genesis_validator.clone());
        genesis_wallet.add_funds(genesis_funds);
        genesis_wallet.set_stake(1);
        wallets.insert(genesis_validator, genesis_wallet);

        Self {
            capacity,
            pending_transactions: BTreeMap::new(),
            public_key,
            private_key,
            blockchain: vec![Signed::new_invalid(genesis_block)],
            wallets,
            network: Box::new(network),
        }
    }

    fn next_validator(&self) -> PublicKey {
        // TODO: use the hash of the last block
        let mut rng = StdRng::seed_from_u64(self.blockchain.len() as u64);
        // Construct the ballot from the current set of
        let total_stake: u64 = self.wallets.values().map(|w| w.staked_amount()).sum();
        assert!(total_stake > 0, "no stakers, BlockChat is doomed");

        let mut winner = rng.gen_range(0..total_stake);
        self.wallets
            .values()
            .find_map(|wallet| {
                if wallet.staked_amount() > winner {
                    Some(wallet.public_key.clone())
                } else {
                    winner -= wallet.staked_amount();
                    None
                }
            })
            .unwrap()
    }

    /// Adds a transaction in the set of pending transactions
    pub fn handle_transaction(&mut self, tx: Signed<Transaction>) {
        let signer = tx.data.sender_address.clone();
        let Ok(tx) = signer.verify(tx) else {
            return;
        };
        self.pending_transactions
            .insert((signer, tx.data.nonce), tx);
        // 2. Validate that there is enough balance
    }

    /// Broadcasts a transaction to the network
    pub fn broadcast_transaction(&mut self, tx: Signed<Transaction>) {
        self.handle_transaction(tx.clone());
        self.network.send(&Message::Transaction(tx));
    }

    /// Broadcasts a block to the network
    fn broadcast_block(&mut self, block: Signed<Block>) {
        self.handle_block(block.clone())
            .expect("minted block was invalid");
        self.network.send(&Message::Block(block));
    }

    /// Attempts to append the given block to the tip of the maintained blockchain. Returns an
    /// error if the block is invalid.
    pub fn handle_block(&mut self, block: Signed<Block>) -> Result<()> {
        // The block must be correctly signed
        let validator = block.data.validator.clone();
        let block = validator.verify(block)?;

        // TODO: Keep out-of-order blocks as pending.

        // The signer must be the expected next validator
        if validator != self.next_validator() {
            return Err(Error::InvalidBlockValidator);
        }

        let mut total_fees = 0;
        let mut new_wallets = self.wallets.clone();
        for tx in block.data.transactions.iter() {
            let sender = tx.data.sender_address.clone();
            let sender_wallet = new_wallets
                .entry(sender.clone())
                .or_insert_with(|| Wallet::with_public_key(sender));

            sender_wallet.apply_tx(tx.clone())?;

            match &tx.data.kind {
                TransactionKind::Coin(_, receiver) | TransactionKind::Message(_, receiver) => {
                    let receiver_wallet = new_wallets
                        .entry(receiver.clone())
                        .or_insert_with(|| Wallet::with_public_key(receiver.clone()));

                    receiver_wallet.apply_tx(tx.clone())?;
                }
                TransactionKind::Stake(_) => {}
            }

            total_fees += tx.data.fees();
        }

        let validator_wallet = new_wallets
            .entry(validator.clone())
            .or_insert_with(|| Wallet::with_public_key(validator));
        validator_wallet.add_funds(total_fees);

        for tx in block.data.transactions.iter() {
            self.pending_transactions
                .remove(&(tx.data.sender_address.clone(), tx.data.nonce));
        }

        self.wallets = new_wallets;
        self.blockchain.push(block);

        Ok(())
    }

    /// Mints a block with at most `capacity` transactions.
    pub fn mint_block(&mut self) -> Signed<Block> {
        let mut tmp_wallets = self.wallets.clone();

        let pending_transactions = std::mem::take(&mut self.pending_transactions);
        let mut transactions = Vec::new();

        for (key, tx) in pending_transactions {
            let sender = tx.data.sender_address.clone();
            let sender_wallet = tmp_wallets
                .entry(sender.clone())
                .or_insert_with(|| Wallet::with_public_key(sender.clone()));

            match sender_wallet.apply_tx(tx.clone()) {
                Err(Error::NonceReused(_, _)) => continue,
                Err(_) => {
                    self.pending_transactions.insert(key, tx);
                    continue;
                }
                Ok(_) => match tx.data.receiver() {
                    Some(receiver) => {
                        let receiver_wallet = tmp_wallets
                            .entry(receiver.clone())
                            .or_insert_with(|| Wallet::with_public_key(receiver.clone()));

                        if sender != receiver {
                            match receiver_wallet.apply_tx(tx.clone()) {
                                Ok(_) => {}
                                Err(_) => {
                                    self.pending_transactions.insert(key, tx);
                                    continue;
                                }
                            }
                        }
                    }
                    None => {}
                },
            }

            if transactions.len() < self.capacity {
                transactions.push(tx);
            }
        }

        let new_block = Block {
            timestamp: Utc::now(),
            transactions,
            validator: self.public_key.clone(),
            parent_hash: self.blockchain.last().unwrap().hash.clone(),
        };

        self.private_key.sign(new_block)
    }

    pub fn step(&mut self) {
        // First handle all pending messages from the network
        // TODO: calculate the duration to the next minting instance.
        self.network.await_events(Some(MINT_INTERVAL));
        while let Some(msg) = self.network.recv() {
            match msg {
                Message::Transaction(tx) => self.handle_transaction(tx),
                Message::Block(block) => {
                    let _ = self.handle_block(block);
                }
            }
        }
        if self.public_key == self.next_validator() {
            let block = self.mint_block();
            self.broadcast_block(block);
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Message {
    Transaction(Signed<Transaction>),
    Block(Signed<Block>),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Block {
    /// The creation timestamp of this block
    // TODO: change this to a type from `chrono`
    timestamp: DateTime<Utc>,
    /// The list of transactions contained in this block.
    transactions: Vec<Signed<Transaction>>,
    /// The public key of the node that minted this block.
    validator: PublicKey,
    /// The hash of the parent block.
    parent_hash: Hash,
}

#[cfg(test)]
mod test {
    use crate::network::TestNetwork;

    use super::*;

    #[test]
    fn basic_test() {
        let (network1, mut network2) = TestNetwork::new();

        let (node_wallet, node_private_key) = Wallet::generate();
        let mut node = Node::new(
            node_wallet.public_key.clone(),
            node_private_key,
            node_wallet.public_key,
            1_000_000,
            5,
            network1,
        );

        // Now create a transaction from a wallet that is not tracked and send it to the node
        let (mut user_wallet, user_key) = Wallet::generate();
        let tx = user_wallet.create_coin_tx(node.public_key.clone(), 42);
        network2.send(&Message::Transaction(user_key.sign(tx)));
        node.step();
        assert_eq!(node.pending_transactions.len(), 1);

        // Now create an invalid transaction and check that it's ignored
        let tx = user_wallet.create_coin_tx(node.public_key.clone(), 42);
        let invalid_tx = Signed::new_invalid(tx);
        network2.send(&Message::Transaction(invalid_tx));
        node.step();
        assert_eq!(node.pending_transactions.len(), 1);
    }

    #[test]
    fn test_mint_block() {
        let (network1, _) = TestNetwork::new();
        let (mut node_wallet, node_private_key) = crate::wallet::test::setup_default_test_wallet();
        let (receiver_wallet, _) = crate::wallet::test::setup_default_test_wallet();

        let mut node = Node::new(
            node_wallet.public_key.clone(),
            node_private_key.clone(),
            node_wallet.public_key.clone(),
            1_000_000,
            5,
            network1,
        );

        const TRANSACTION_COUNT: usize = 7;
        let coin_amount = 1000;
        let mut transactions = Vec::new();

        // Apply more transactions than the block capacity
        for _ in 0..TRANSACTION_COUNT {
            let tx = node_wallet
                .clone()
                .create_coin_tx(receiver_wallet.clone().public_key, coin_amount);
            let signed_tx = node_private_key.sign(tx.clone());

            node_wallet.apply_tx(signed_tx.clone()).unwrap();
            node.handle_transaction(signed_tx.clone());

            if transactions.len() < node.capacity {
                transactions.push(signed_tx);
            }
        }

        let block = node.mint_block();
        assert_eq!(block.data.transactions.len(), 5);
        assert_eq!(block.data.transactions, transactions);
        assert_eq!(block.data.validator, node_wallet.public_key);
        assert_eq!(block.data.parent_hash, node.blockchain[0].hash);
    }
}

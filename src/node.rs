// Remove this when it's not a WIP
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::time::SystemTime;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::crypto::{Hash, PrivateKey, PublicKey, Signed};
use crate::error::{Error, Result};
use crate::network::Network;
use crate::wallet::{Transaction, TransactionKind, Wallet};

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

impl Node {
    pub fn new(
        public_key: PublicKey,
        private_key: PrivateKey,
        blockchain: Vec<Signed<Block>>,
        capacity: usize,
        network: impl Network<Message> + 'static,
    ) -> Self {
        Self {
            capacity,
            pending_transactions: BTreeMap::new(),
            public_key,
            private_key,
            blockchain,
            wallets: BTreeMap::new(),
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
    fn handle_transaction(&mut self, tx: Signed<Transaction>) {
        let signer = tx.data.sender_address.clone();
        let Ok(tx) = signer.verify(tx) else {
            return;
        };
        self.pending_transactions
            .insert((signer, tx.data.nonce), tx);
        // 2. Validate that there is enough balance
    }

    /// Attempts to append the given block to the tip of the maintained blockchain. Returns an
    /// error if the block is invalid.
    fn handle_block(&mut self, block: Signed<Block>) -> Result<()> {
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
    fn mint_block(&mut self) -> Signed<Block> {
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
                        match receiver_wallet.apply_tx(tx.clone()) {
                            Ok(_) => {}
                            Err(_) => {
                                self.pending_transactions.insert(key, tx);
                                continue;
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
            timestamp: SystemTime::now(),
            transactions,
            validator: self.public_key.clone(),
            parent_hash: Hash,
            // TODO: Fix the parent hash.
        };

        self.private_key.sign(new_block)
    }

    fn step(&mut self) {
        // First handle all pending messages from the network
        self.network.await_events(None);
        while let Some(msg) = self.network.recv() {
            match msg {
                Message::Transaction(tx) => self.handle_transaction(tx),
                Message::Block(block) => {
                    let _ = self.handle_block(block);
                }
            }
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
    timestamp: SystemTime,
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
            node_wallet.public_key,
            node_private_key,
            vec![],
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
}

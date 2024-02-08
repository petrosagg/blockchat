use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Receiver, Sender};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::crypto::{Hash, PrivateKey, PublicKey, Signature, Signed};
use crate::error::Result;
use crate::wallet::{Transaction, Wallet};

pub struct Node {
    /// The maximum number of transactions contained in each block.
    capacity: usize,
    /// The set of signed but not necessarily valid transactions waiting to be included in a block.
    pending_transactions: HashSet<Signed<Transaction>>,
    /// The current blockchain.
    blockchain: Vec<Signed<Block>>,
    /// The wallet of this node.
    wallet: Wallet,
    /// The balances per public key.
    balances: HashMap<PublicKey, u64>,
    /// The stake amounts per public key.
    stake_pool: HashMap<PublicKey, u64>,
    /// A receiver of messages from someone in the network.
    rx: Receiver<Message>,
    /// A sender of messages that will be broadcasted to everyone.
    tx: Sender<Message>,
}

impl Node {
    fn new(
        wallet: Wallet,
        blockchain: Vec<Signed<Block>>,
        capacity: usize,
        tx: Sender<Message>,
        rx: Receiver<Message>,
    ) -> Self {
        Self {
            capacity,
            pending_transactions: HashSet::new(),
            blockchain,
            wallet,
            // Calculate the balances based on the provided blockchain
            balances: HashMap::new(),
            // Calculate the stake pool based on the provided blockchain
            stake_pool: HashMap::new(),
            rx,
            tx,
        }
    }

    /// Adds a transaction in the set of pending transactions
    fn handle_transaction(&mut self, tx: Signed<Transaction>) {
        // 1. Validate signature
        // 2. Validate that there is enough balance
        self.pending_transactions.insert(tx);
    }

    /// Attempts to append the given block to the tip of the maintained blockchain. Returns an
    /// error if the block is invalid.
    fn handle_block(&mut self, block: Signed<Block>) {
        // 1. validate that this block came from the leader and contains valid transactions.
        // 2. remove transactions referenced in the block from pending transactions
        // 3. update stake pool and wallet state
        todo!()
    }

    /// Mints a block with at most `capacity` transactions.
    fn mint_block(&mut self) -> Signed<Block> {
        todo!()
    }

    fn step(&mut self) {
        // First handle all pending messages from the network
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Message::Transaction(tx) => self.handle_transaction(tx),
                Message::Block(block) => self.handle_block(block),
            }
        }
        // If the pending block has reached `capacity` transactions, mint it and broadcast it.
    }
}

pub enum Message {
    Transaction(Signed<Transaction>),
    Block(Signed<Block>),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
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

#[derive(Clone, Default)]
pub struct StakeState {
    stake: u64,
}

#[cfg(test)]
mod test {
    use std::sync::mpsc;

    use super::*;

    #[test]
    fn basic_test() {
        let (tx, node_rx) = mpsc::channel();
        let (node_tx, rx) = mpsc::channel();

        let node_wallet = Wallet::new();
        let mut node = Node::new(node_wallet, vec![], 5, node_tx, node_rx);
        node.step();

        // Now create a transaction from a wallet that is not tracked and send it to the node
        let mut user_wallet = Wallet::new();
        let transaction = user_wallet.sign_coin_transaction(&node.wallet.public_key, 42);
        let _ = tx.send(Message::Transaction(transaction));

        // Now we step the node which should observe the transaction and ignore it.
        node.step();
        assert!(node.pending_transactions.is_empty());
    }
}

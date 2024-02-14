// Remove this when it's not a WIP
#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::crypto::{Hash, PublicKey, Signed};
use crate::network::Network;
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
    /// This node's handle to the network
    network: Box<dyn Network<Message>>,
}

impl Node {
    fn new(
        wallet: Wallet,
        blockchain: Vec<Signed<Block>>,
        capacity: usize,
        network: impl Network<Message> + 'static,
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
            network: Box::new(network),
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
    fn handle_block(&mut self, _block: Signed<Block>) {
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
        self.network.await_events();
        while let Some(msg) = self.network.recv() {
            match msg {
                Message::Transaction(tx) => self.handle_transaction(tx),
                Message::Block(block) => self.handle_block(block),
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
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

    use crate::network::TestNetwork;

    use super::*;

    #[test]
    fn basic_test() {
        let (network1, mut network2) = TestNetwork::new();

        let node_wallet = Wallet::new();
        let mut node = Node::new(node_wallet, vec![], 5, network1);

        // Now create a transaction from a wallet that is not tracked and send it to the node
        let mut user_wallet = Wallet::new();
        let transaction = user_wallet.sign_coin_transaction(&node.wallet.public_key, 42);
        network2.send(&Message::Transaction(transaction));

        node.step();
        assert_eq!(node.pending_transactions.len(), 1);
    }
}

use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use chrono::{DateTime, Utc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::crypto::{Address, Hash, PrivateKey, PublicKey, Signed};
use crate::error::{Error, Result};
use crate::network::Network;
use crate::wallet::{Transaction, TransactionKind, Wallet};

const MINT_INTERVAL: Duration = Duration::from_secs(1);

pub struct Node {
    // The name of this node. Used for logging
    name: String,
    /// The maximum number of transactions contained in each block.
    capacity: usize,
    /// The set of signed but not necessarily valid transactions waiting to be included in a block.
    pending_transactions: BTreeMap<(Address, u64), Signed<Transaction>>,
    /// The current blockchain.
    blockchain: Vec<Signed<Block>>,
    /// The public key of the wallet of this node.
    address: Address,
    /// The public key of the wallet of this node.
    public_key: PublicKey,
    /// The private key of the wallet of this node.
    private_key: PrivateKey,
    /// The wallet of this node that can be manipulated independencly of the block minting process
    /// to create transactions.
    node_wallet: Wallet,
    /// The state of each known wallet indexed by public key. We use a BTreeMap to always maintain
    /// the wallets in sorted public key order which helps perform the validator election.
    wallets: BTreeMap<Address, Wallet>,
    /// Messages that should be broadcast on the next tick
    outbox: Vec<Message>,
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
            .field("node_wallet", &self.node_wallet)
            .finish()
    }
}

impl Node {
    pub fn new(
        name: String,
        public_key: PublicKey,
        private_key: PrivateKey,
        genesis_validator: PublicKey,
        genesis_funds: u64,
        capacity: usize,
    ) -> Self {
        let mut wallets = BTreeMap::new();
        let node_address = Address::from_public_key(&public_key);
        let node_wallet = Wallet::from_address(node_address.clone());
        wallets.insert(node_address.clone(), node_wallet.clone());

        let genesis_address = Address::from_public_key(&genesis_validator);
        let genesis_tx = Transaction {
            sender_address: Address::invalid(),
            kind: TransactionKind::Coin(genesis_funds, genesis_address.clone()),
            nonce: 0,
        };

        let genesis_block = Block {
            timestamp: DateTime::<Utc>::MIN_UTC,
            transactions: vec![Signed::new_invalid(genesis_tx)],
            validator: Address::invalid(),
            parent_hash: "0000000000000000000000000000000000000000000000000000000000000001"
                .parse()
                .unwrap(),
        };

        let genesis_wallet = wallets
            .entry(genesis_address.clone())
            .or_insert_with(|| Wallet::from_address(genesis_address));
        genesis_wallet.add_funds(genesis_funds);
        genesis_wallet.set_stake(1);

        Self {
            name,
            capacity,
            pending_transactions: BTreeMap::new(),
            node_wallet: wallets[&node_address].clone(),
            address: node_address,
            public_key,
            private_key,
            blockchain: vec![Signed::new_invalid(genesis_block)],
            wallets,
            outbox: vec![],
        }
    }

    fn next_validator(&self) -> Address {
        let seed = self.blockchain.last().unwrap().hash.0;
        let mut rng = StdRng::from_seed(seed);
        // Construct the ballot from the current set of
        let total_stake: u64 = self.wallets.values().map(|w| w.staked_amount()).sum();
        assert!(total_stake > 0, "no stakers, BlockChat is doomed");

        let mut winner = rng.gen_range(0..total_stake);
        self.wallets
            .values()
            .find_map(|wallet| {
                if wallet.staked_amount() > winner {
                    Some(wallet.address.clone())
                } else {
                    winner -= wallet.staked_amount();
                    None
                }
            })
            .unwrap()
    }

    /// The address of this node's wallet.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// This node's wallet.
    pub fn wallet(&self) -> &Wallet {
        &self.node_wallet
    }

    /// This node's wallet.
    pub fn wallet_mut(&mut self) -> &mut Wallet {
        &mut self.node_wallet
    }

    pub fn blockchain(&self) -> &[Signed<Block>] {
        &self.blockchain
    }

    pub fn total_transactions(&self) -> usize {
        self.blockchain
            .iter()
            .map(|block| block.data.transactions.len())
            .sum()
    }

    /// Reports whether this node is aware of non-confirmed transactions
    pub fn has_pending_transactions(&self) -> bool {
        !self.pending_transactions.is_empty()
    }

    /// Adds a transaction in the set of pending transactions
    pub fn handle_transaction(&mut self, tx: Signed<Transaction>) -> Result<()> {
        let signer = tx.data.sender_address.clone();
        tx.verify()?;
        self.pending_transactions
            .insert((signer, tx.data.nonce), tx);
        // 2. Validate that there is enough balance
        Ok(())
    }

    /// Attempts to append the given block to the tip of the maintained blockchain. Returns an
    /// error if the block is invalid.
    pub fn handle_block(&mut self, block: Signed<Block>) -> Result<()> {
        log::trace!(
            "{}: handling block containing {} transactions",
            self.name,
            block.data.transactions.len()
        );
        // The block must be correctly signed
        block.verify()?;

        // TODO: Keep out-of-order blocks as pending.

        // The signer must be the expected next validator
        let validator = block.data.validator.clone();
        if validator != self.next_validator() {
            return Err(Error::InvalidBlockValidator);
        }

        let mut total_fees = 0;
        let mut new_wallets = self.wallets.clone();
        for tx in block.data.transactions.iter() {
            let sender = tx.data.sender_address.clone();
            let sender_wallet = new_wallets
                .entry(sender.clone())
                .or_insert_with(|| Wallet::from_address(sender.clone()));

            sender_wallet.apply_tx(tx.clone())?;

            match &tx.data.kind {
                TransactionKind::Coin(_, receiver) | TransactionKind::Message(_, receiver) => {
                    let receiver_wallet = new_wallets
                        .entry(receiver.clone())
                        .or_insert_with(|| Wallet::from_address(receiver.clone()));

                    receiver_wallet.apply_tx(tx.clone())?;
                    if receiver == &self.address {
                        self.node_wallet.apply_tx(tx.clone())?;
                    }
                }
                TransactionKind::Stake(_) => {}
            }

            total_fees += tx.data.fees();
        }

        let validator_wallet = new_wallets
            .entry(validator.clone())
            .or_insert_with(|| Wallet::from_address(validator.clone()));
        validator_wallet.add_funds(total_fees);
        if validator == self.address {
            self.node_wallet.add_funds(total_fees);
        }

        for tx in block.data.transactions.iter() {
            log::trace!("{}: accepted valid tx {:?}", self.name, tx.hash);
            self.pending_transactions
                .remove(&(tx.data.sender_address.clone(), tx.data.nonce));
        }

        self.wallets = new_wallets;
        log::info!("{}: accepted valid block {:?}", self.name, block.hash);
        self.blockchain.push(block);

        Ok(())
    }

    /// Mints a block with at most `capacity` transactions.
    pub fn mint_block(&mut self) -> Signed<Block> {
        let mut tmp_wallets = self.wallets.clone();

        let pending_transactions = std::mem::take(&mut self.pending_transactions);
        let mut transactions = Vec::new();

        for (key, tx) in pending_transactions {
            if transactions.len() < self.capacity {
                let sender = tx.data.sender_address.clone();
                let sender_wallet = tmp_wallets
                    .entry(sender.clone())
                    .or_insert_with(|| Wallet::from_address(sender.clone()));

                match sender_wallet.apply_tx(tx.clone()) {
                    Err(err @ Error::NonceReused(_, _)) => {
                        log::trace!("{}: dropping invalid tx {:?}: {err}", self.name, tx.hash);
                        continue;
                    }
                    Err(_) => {
                        self.pending_transactions.insert(key, tx);
                        continue;
                    }
                    Ok(_) => match tx.data.receiver() {
                        Some(receiver) => {
                            let receiver_wallet = tmp_wallets
                                .entry(receiver.clone())
                                .or_insert_with(|| Wallet::from_address(receiver.clone()));

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

                transactions.push(tx);
            } else {
                self.pending_transactions.insert(key, tx);
            }
        }

        let new_block = Block {
            timestamp: Utc::now(),
            transactions,
            validator: Address::from_public_key(&self.public_key),
            parent_hash: self.blockchain.last().unwrap().hash.clone(),
        };

        self.private_key.sign(new_block)
    }

    pub fn sign_transaction(&self, tx: Transaction) -> Signed<Transaction> {
        self.private_key.sign(tx)
    }

    /// Broadcasts a transaction to the network
    pub fn broadcast_transaction(&mut self, tx: Signed<Transaction>) {
        log::trace!(
            "{}: broadcasting tx {:?}: {:?}",
            self.name,
            tx.hash,
            tx.data
        );
        if let Err(err) = self.handle_transaction(tx.clone()) {
            log::warn!("{}: broadcasting invalid transaction {err}", self.name);
        }
        self.outbox.push(Message::Transaction(tx));
    }

    /// Broadcasts a block to the network
    pub fn broadcast_block(&mut self, block: Signed<Block>) {
        if let Err(err) = self.handle_block(block.clone()) {
            log::warn!("{}: broadcasting invalid block {err}", self.name);
        }
        self.outbox.push(Message::Block(block));
    }

    pub fn step<N: Network<Message>>(&mut self, network: &mut N) -> Option<Duration> {
        // First send all outstanding messages to the network
        for message in self.outbox.drain(..) {
            network.send(&message);
        }

        // Then handle all pending messages from the network
        while let Some(msg) = network.recv() {
            match msg {
                Message::Transaction(tx) => match self.handle_transaction(tx) {
                    Ok(_) => {}
                    Err(err) => log::info!("{}: rejected invalid transaction {err}", self.name),
                },
                Message::Block(block) => match self.handle_block(block) {
                    Ok(_) => {}
                    Err(err) => log::info!("{}: rejected invalid block {err}", self.name),
                },
            }
        }

        // TODO: this might have to run multiple times per step if this node is the validator of
        // the next block too.
        if self.address == self.next_validator() {
            let last_block_ts = self.blockchain().last().unwrap().data.timestamp;
            let next_block_ts = last_block_ts + MINT_INTERVAL;
            // A new block is minted if we have enough pending transaction to create a full block
            // or if enough time has passed from the previous mint.
            if Utc::now() > next_block_ts || self.pending_transactions.len() >= self.capacity {
                let block = self.mint_block();
                log::info!("{}: broadcasting minted block {:?}", self.name, block.hash);
                self.handle_block(block.clone())
                    .expect("minted block was invalid");
                network.send(&Message::Block(block));
                if self.address == self.next_validator() {
                    Some(MINT_INTERVAL)
                } else {
                    None
                }
            } else {
                Some((next_block_ts - Utc::now()).to_std().unwrap())
            }
        } else {
            None
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
    pub timestamp: DateTime<Utc>,
    /// The list of transactions contained in this block.
    pub transactions: Vec<Signed<Transaction>>,
    /// The public key of the node that minted this block.
    pub validator: Address,
    /// The hash of the parent block.
    pub parent_hash: Hash,
}

#[cfg(test)]
mod test {
    use crate::{crypto, network::TestNetwork};

    use super::*;

    #[test]
    fn basic_test() {
        let (mut network1, mut network2) = TestNetwork::new();

        let (node_private_key, node_public_key) = crypto::generate_keypair();
        let mut node = Node::new(
            "test_node".into(),
            node_public_key.clone(),
            node_private_key,
            node_public_key,
            1_000_000,
            5,
        );

        // Now create a transaction from a wallet that is not tracked and send it to the node
        let (user_key, user_public_key) = crypto::generate_keypair();
        let user_wallet = Wallet::from_public_key(&user_public_key);
        let tx = user_wallet.create_coin_tx(Address::from_public_key(&node.public_key), 42);
        network2.send(&Message::Transaction(user_key.sign(tx)));
        node.step(&mut network1);
        assert_eq!(node.pending_transactions.len(), 1);

        // Now create an invalid transaction and check that it's ignored
        let tx = user_wallet.create_coin_tx(Address::from_public_key(&node.public_key), 42);
        let invalid_tx = Signed::new_invalid(tx);
        network2.send(&Message::Transaction(invalid_tx));
        node.step(&mut network1);
        assert_eq!(node.pending_transactions.len(), 1);
    }

    #[test]
    fn test_mint_block() {
        let (mut node_wallet, node_public_key, node_private_key) =
            crate::wallet::test::setup_default_test_wallet();
        let (receiver_wallet, _, _) = crate::wallet::test::setup_default_test_wallet();

        let mut node = Node::new(
            "test_node".into(),
            node_public_key.clone(),
            node_private_key.clone(),
            node_public_key.clone(),
            1_000_000,
            5,
        );

        const TRANSACTION_COUNT: usize = 7;
        let coin_amount = 1000;
        let mut transactions = Vec::new();

        // Apply more transactions than the block capacity
        for _ in 0..TRANSACTION_COUNT {
            let tx = node_wallet
                .clone()
                .create_coin_tx(receiver_wallet.address.clone(), coin_amount);
            let signed_tx = node_private_key.sign(tx.clone());

            node_wallet.apply_tx(signed_tx.clone()).unwrap();
            node.handle_transaction(signed_tx.clone()).unwrap();

            if transactions.len() < node.capacity {
                transactions.push(signed_tx);
            }
        }

        let block = node.mint_block();
        assert_eq!(block.data.transactions.len(), 5);
        assert_eq!(block.data.transactions, transactions);
        assert_eq!(block.data.validator, node_wallet.address);
        assert_eq!(block.data.parent_hash, node.blockchain[0].hash);
    }
}

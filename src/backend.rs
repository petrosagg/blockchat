use std::collections::{HashSet, HashMap};
use std::sync::mpsc::{Receiver, Sender};
use std::time::SystemTime;

use serde::{Serialize, Deserialize};

use crate::crypto::{PublicKey, PrivateKey, Signature, Hash};
use crate::error::Result;

pub struct Node {
    id: usize,
    /// The number of transactions contained in each block.
    capacity: usize,
    /// The set of signed but not necessarily valid transactions waiting to be included in a block.
    pending_transactions: HashSet<SignedTransaction>,
    /// The current blockchain.
    blockchain: Vec<Block>,
    /// The public key of this wallet.
    stake_state: Vec<StakeState>,
    /// The private key of this wallet.
    private_key: PrivateKey,
    wallet_state: HashMap<PublicKey, WalletState>,
    rx: Receiver<Message>,
    tx: Sender<Message>,
}

impl Node {
    /// Adds a transaction in the set of pending transactions
    fn receive_transaction(&mut self) {
    }

    /// Attempts to append the given block to the tip of the maintained blockchain. Returns an
    /// error if the block is invalid.
    fn append_block(&mut self) -> Result<()> {
        todo!()
    }

    /// Mints a block by waitng for `capacity` valid transactions to appear.
   fn mint_block(&mut self) -> Result<Block> {
       // 1. Take at most `self.capacity` valid txs from current set
       // 2. Wait for more txs from the network if current set was insufficient
       // 3. Sign new block 
       todo!()
    }

   /// Validates the proposed block against the current tip
   fn validate_block(&mut self, block: &Block) -> Result<()> {
       todo!()
   }
}

pub enum Message {
    Transaction(SignedTransaction),
    Block(SignedBlock),
}

pub struct StakeState {
    public_key: PublicKey,
    stake: u64,
}

pub struct WalletState {
    balance: u64,
}

pub struct Wallet {
    /// The public key of this wallet.
    pub public_key: PublicKey,
    /// The private key of this wallet.
    private_key: PrivateKey,
    /// An auto-increment nonce used to sign transactions.
    nonce: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct SignedBlock {
    /// The hash of the block.
    hash: Hash,
    /// The signature of the block.
    signature: Signature,
    // The actual block data.
    block: Block,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct Block {
    /// The auto-increment index of this block.
    index: u64,
    /// The creation timestamp of this block
    // TODO: change this to a type from `chrono`
    timestamp: SystemTime,
    /// The list of transactions contained in this block.
    transactions: Vec<SignedTransaction>,
    /// The public key of the node that minted this block.
    validator: PublicKey,
    /// The hash of the parent block.
    previous_hash: Hash,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// The hash of this transaction.
    hash: Hash,
    /// A signature proving that the sender wallet created this transaction.
    signature: Signature,
    /// The tx data being signed,
    tx: Transaction,
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
    nonce: usize,
}

impl Wallet {
    fn new() -> Self {
        todo!()
    }

    fn create_coin_transaction(&self, /* args */) -> Transaction {
        todo!()
    }

    fn create_message_transaction(&self, /* args */) -> Transaction {
        todo!()
    }

    fn broadcast_transaction(&self, tx: SignedTransaction) {
    }


    fn sign_transaction(&self, tx: Transaction) -> SignedTransaction {
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
    // TODO: consider adding a stake variant to record stake state in the blockchain
    
}


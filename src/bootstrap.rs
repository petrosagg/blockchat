//! Routines for bootstrapping a blockchat network of a given configuration.
//!

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::backend::Block;
use crate::crypto::Signed;
use crate::error::Result;
use crate::wallet::Wallet;
use crate::{backend::Node, crypto::PublicKey};

pub struct BootstrapNode {
    /// The capacity per block.
    capacity: usize,
    // The number of expected nodes in the system.
    peers: usize,
    /// The socket address of the bootstrap leader.
    bootstrap_addr: SocketAddr,
    /// The wallet of this node.
    wallet: Wallet,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
/// The initial state of the blockchain that is exchanged during bootstrapping.
struct BootstrapState {
    /// The list of all the peers with their associated socket addresses and public keys.
    peer_info: Vec<PeerInfo>,
    /// The current blockchain.
    blockchain: Vec<Signed<Block>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PeerInfo {
    /// The listen address of this peer.
    listen_addr: SocketAddr,
    /// The public key of this peer's wallet.
    public_key: PublicKey,
}

/// A wrapper over a TCP connection that is able to send and receive data using line delimited
/// JSON.
struct TypedJsonStream {
    /// The underlying TCP stream.
    stream: BufReader<TcpStream>,
    /// A temporary buffer to hold the JSON data before decoding.
    buf: String,
}

impl TypedJsonStream {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufReader::new(stream),
            buf: String::new(),
        }
    }

    fn send<S: Serialize>(&mut self, msg: &S) {
        serde_json::to_writer(self.stream.get_mut(), &msg).unwrap();
        self.stream.get_mut().write(&[b'\n']).unwrap();
        self.stream.get_mut().flush().unwrap();
    }

    fn recv<D: DeserializeOwned>(&mut self) -> D {
        self.buf.clear();
        self.stream.read_line(&mut self.buf).unwrap();
        serde_json::from_str(&self.buf).unwrap()
    }
}

impl BootstrapNode {
    /// Bootstraps this node and instantiates a fully configured `Node`.
    fn bootstrap(self) -> Node {
        // let listener = TcpListener::bind(self.listen_addr).unwrap();
        // if self.listen_addr == self.bootstrap_addr {
        //     let listener = listener.try_clone().unwrap();
        //     self.spawn_bootstrap_helper(listener)
        // }
        //
        // // First we connect to the bootstrap server to learn the initial state
        // let stream = TcpStream::connect(self.bootstrap_addr).unwrap();
        // let mut bootstrap_helper = TypedJsonStream::new(stream);

        // let info = PeerInfo {
        //     listen_addr: self.listen_addr,
        //     public_key: self.wallet.public_key,
        // };
        // bootstrap_helper.send(&info);

        // let initial_state = bootstrap_helper.recv();
        // let index = initial_state.index;
        // // TODO: validate initial blockchain

        // let streams = Vec::with_capacity(self.peers);
        // // Seed the streams with the bootstrap node
        // streams.push(bootstrap_helper);

        // // Establish a TCP socket with every node whose ID is strictly less than ours
        // for info in 1..index {
        //     let stream = TcpStream::connect(info.peer_addr).unwrap();
        //     let node = TypedJsonStream::new(stream);
        //     streams.push(node);
        // }

        // // Now start a TCP listener to accept connections from every node whose ID is strictly
        // // greater than ours
        // let listener = TcpListener::bind(self.listen_addr).unwrap();
        // for index in (index + 1)..self.peers {
        //     // Accept a connection and receive this peer's info
        //     let mut stream = TypedJsonStream::new(listener.accept().unwrap().0);
        //     let info = stream.recv();
        //     peer_info.push(info);
        //     streams.push(stream);
        // }

        // //
        // // 0 1 2 3 4 5
        // //
        // // 0 <- 1, 2, 3, 4, 5
        // // 1 <- 2, 3, 4, 5
        // // 2 <- 3, 4, 5
        // // 3 <- 4, 5
        // // 4 <- 5
        todo!()
    }

    fn spawn_bootstrap_helper(&self, listener: TcpListener) -> std::thread::JoinHandle<()> {
        // // If we are the bootstrap server we'll spawn a tiny helper thread whose sole purpose
        // // is to collect all the listen addresses and assign a unique index to them.
        // let peers = self.peers;
        // std::thread::spawn(move || {
        //     let addrs = Vec::with_capacity(peers);
        //     for index in 0..peers {
        //         let mut stream = TypedJsonStream::new(listener.accept().unwrap().0);
        //         let info = stream.recv();
        //         peer_info.push(info);
        //         streams.push(stream);
        //     }
        //     // Now let each peer learn about every other peer.
        //     for peer_stream in streams.iter_mut() {
        //         peer_stream.send(&initial_state);
        //     }
        // });
        // todo!()
        todo!()
    }

    /// Runs the boostrap logic assuming this node is the leader.
    fn bootstrap_leader(self) -> Node {
        // let mut peer_info = Vec::with_capacity(self.peers);
        // // Seed the peer info with our information
        // peer_info.push(PeerInfo {
        //     listen_addr: self.listen_addr,
        //     public_key: self.wallet.public_key,
        // });
        //
        // // Receive the info from the rest of the peers
        // let mut streams = Vec::with_capacity(self.peers);
        // let listener = TcpListener::bind(self.listen_addr).unwrap();
        // for index in 1..self.peers {
        //     // Accept a connection and receive this peer's info
        //     let mut stream = TypedJsonStream::new(listener.accept().unwrap().0);
        //     let info = stream.recv();
        //     peer_info.push(info);
        //     streams.push(stream);
        // }

        // let initial_state = BootstrapState {
        //     peer_info,
        //     // TODO: fill in the genesis block
        //     blockchain: vec![],
        // };

        // // Now let each peer learn about every other peer.
        // for peer_stream in streams.iter_mut() {
        //     peer_stream.send(&initial_state);
        // }
        // todo!()

        // // Node {
        // //     id: 0,
        // //     capacity: self.capacity,
        // //     pending_transactions: HashSet::new(),
        // //     blockchain: initial_state.blockchain,
        // //     /// TODO: Derive the initial stake state from the blockchain if that is stored there.
        // //     stake_state: vec![StakeState::default(); self.peers],
        todo!()
    }
}

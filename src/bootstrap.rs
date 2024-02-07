//! Routines for bootstrapping a blockchat network of a given configuration.
//!
//!

use std::io::{Write, BufReader, BufRead};
use std::net::{SocketAddr, TcpStream, TcpListener};

use serde::{Serialize, Deserialize};

use crate::backend::{SignedBlock, SignedTransaction, Wallet};
use crate::{crypto::PublicKey, backend::Node};
use crate::error::Result;

pub struct BootstrapNode {
    /// Whether this node leads the bootstrap phase, i.e whether this is node 0.
    is_leader: bool,
    /// The capacity per block.
    capacity: usize,
    // The number of expected nodes in the system
    peers: usize,
    /// The socket address of this bootstrap node should listen to.
    listen_addr: SocketAddr,
    /// The wallet of this node.
    wallet: Wallet,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Message {
    Hello(PeerInfo),
    BootstrapComplete(Vec<PeerInfo>, Vec<SignedBlock>),
    Transaction(SignedTransaction),
    Block(SignedBlock),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PeerInfo {
    /// The listen address of this peer.
    listen_addr: SocketAddr,
    /// The public key of this peer's wallet.
    public_key: PublicKey,
}

struct MessageStream {
    stream: BufReader<TcpStream>,
    buf: String,
}

impl MessageStream {
    fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufReader::new(stream),
            buf: String::new(),
        }
    }

    fn send(&mut self, msg: &Message) {
        serde_json::to_writer(self.stream.get_mut(), &msg).unwrap();
        self.stream.get_mut().write(&[b'\n']).unwrap();
        self.stream.get_mut().flush().unwrap();
    }

    fn recv(&mut self) -> Message {
        self.buf.clear();
        self.stream.read_line(&mut self.buf).unwrap();
        serde_json::from_str(&self.buf).unwrap()
    }
}

impl BootstrapNode {
    /// Bootstraps this node and instantiates a fully configured `Node`.
    fn bootstrap(self) -> Node {
        if self.is_leader {
            self.bootstrap_leader()
        } else {
            self.bootstrap_follower()
        }
    }

    /// Runs the boostrap logic assuming this node is the leader.
    fn bootstrap_leader(self) -> Node {
        let mut peer_info = Vec::with_capacity(self.peers);
        // Seed the peer info with our information
        peer_info.push(PeerInfo {
            listen_addr: self.listen_addr,
            public_key: self.wallet.public_key,
        });
        
        // Receive the info from the rest of the peers
        let mut streams = Vec::with_capacity(self.peers);
        let listener = TcpListener::bind(self.listen_addr).unwrap();
        for index in 1..self.peers {
            // Accept a connection and receive this peer's info
            let mut stream = MessageStream::new(listener.accept().unwrap().0);
            match stream.recv() {
                Message::Hello(_) => todo!(),
                msg => panic!("Unexpected message: {msg:?}"),
            }
            streams.push(stream);
        }

        // Now let each peer learn about every other peer.
        for peer_stream in streams.iter() {
            // broadcast peer_info into stream
        }
        todo!()
    }

    /// Runs the boostrap logic assuming this node is a follower.
    fn bootstrap_follower(self) -> Node {
        todo!()
        // // Should connect to bootstrap host/port
        // let mut stream = TcpStream::connect((args.bootstrap_host, args.bootstrap_port))?;
        // let mut stream = BufReader::new(stream);

        // let command = NodeCommand::Bootstrap;
        // let bytes = serde_json::to_writer(stream.get_mut(), &command);
        // stream.get_mut().write(&[b'\n'])?;
        // stream.get_mut().flush()?;

        // let mut response_bytes = String::new();
        // if stream.read_line(&mut response_bytes).unwrap() > 0 {
        //     let response: NodeResponse = serde_json::from_str(&response_bytes)?;
        //     println!("Bootstrap successful, received response {response:?}")
        // }

        // std::thread::sleep(Duration::from_secs(60));
    }
}

// Remove this when it's not a WIP
#![allow(dead_code)]

//! Routines for bootstrapping a blockchat network of a given configuration.

use std::net::{IpAddr, SocketAddr, TcpListener};

use serde::{Deserialize, Serialize};

use crate::crypto::{Address, PrivateKey, PublicKey};
use crate::network::broadcast::Broadcaster;
use crate::network::discovery::{bootstrap_helper, discover_peers};
use crate::network::Network;
use crate::node::{Message, Node};
use crate::wallet::Wallet;

const GENESIS_FUNDS_PER_NODE: u64 = 1000;

pub struct BootstrapConfig {
    /// Whether this node is responsible for running the bootstrap helper
    pub bootstrap_leader: bool,
    /// The capacity per block.
    pub capacity: usize,
    // The number of expected nodes in the system.
    pub peers: usize,
    /// The socket address of the bootstrap helper.
    pub bootstrap_addr: SocketAddr,
    /// The socket address this node should listen to.
    pub listen_ip: IpAddr,
    /// The public_key of this node.
    pub public_key: PublicKey,
    /// The private key of this node.
    pub private_key: PrivateKey,
}

/// The peer info exchanged during discovery.
#[derive(Serialize, Deserialize)]
struct PeerInfo {
    /// The socket address the peer will listen on.
    listen_addr: SocketAddr,
    /// The public key of this peer.
    public_key: PublicKey,
}

pub fn bootstrap(config: BootstrapConfig) -> (Node, Broadcaster<Message>) {
    if config.bootstrap_leader {
        let genesis_validator = config.public_key.clone();
        std::thread::spawn(move || {
            bootstrap_helper::<PeerInfo, _>(config.bootstrap_addr, config.peers, genesis_validator)
        });
    }

    let listener = TcpListener::bind((config.listen_ip, 0)).unwrap();

    let peer_info = PeerInfo {
        listen_addr: listener.local_addr().unwrap(),
        public_key: config.public_key.clone(),
    };
    let (my_index, peer_infos, genesis_validator) =
        discover_peers::<PeerInfo, PublicKey>(config.bootstrap_addr, peer_info);

    let peer_addrs: Vec<_> = peer_infos.iter().map(|info| info.listen_addr).collect();
    let mut network = Broadcaster::<Message>::new(listener, &peer_addrs, my_index);

    let genesis_funds = GENESIS_FUNDS_PER_NODE * (config.peers as u64);

    let node = Node::new(
        format!("node-{my_index}"),
        config.public_key,
        config.private_key.clone(),
        genesis_validator.clone(),
        genesis_funds,
        config.capacity,
    );

    if config.bootstrap_leader {
        let mut genesis_wallet = Wallet::from_public_key(&genesis_validator);
        genesis_wallet.add_funds(genesis_funds);
        for peer_info in peer_infos {
            // No need to seed the genesis wallet.
            if peer_info.public_key == genesis_validator {
                continue;
            }
            let tx = genesis_wallet
                .create_coin_tx(Address::from_public_key(&peer_info.public_key), 1000);
            let signed_tx = config.private_key.sign(tx);
            network.send(&Message::Transaction(signed_tx));
        }
    }

    (node, network)
}

#[cfg(test)]
mod test {
    use crate::crypto;

    use super::*;

    #[test]
    fn bootstrap_small_cluster() {
        tracing_subscriber::fmt().with_test_writer().init();

        let bootstrap_addr = "127.0.0.1:13000".parse().unwrap();
        let listen_ip = "127.0.0.1".parse().unwrap();

        const PEERS: usize = 5;
        const CAPACITY: usize = 5;

        let mut node_handles = vec![];

        // Start threads for the non-leader nodes
        for _ in 1..PEERS {
            let (private_key, public_key) = crypto::generate_keypair();
            let config = BootstrapConfig {
                bootstrap_leader: false,
                capacity: CAPACITY,
                peers: PEERS,
                bootstrap_addr,
                listen_ip,
                public_key,
                private_key,
            };
            let handle = std::thread::spawn(move || {
                let (mut node, mut network) = bootstrap(config);
                loop {
                    let timeout = node.step(&mut network);
                    if node.blockchain().len() > 2 {
                        break;
                    }
                    network.await_events(timeout);
                }
            });
            node_handles.push(handle);
        }

        // Start the leader node and verify its state
        let (private_key, public_key) = crypto::generate_keypair();
        let config = BootstrapConfig {
            bootstrap_leader: true,
            capacity: CAPACITY,
            peers: PEERS,
            bootstrap_addr,
            listen_ip,
            public_key,
            private_key,
        };
        let (mut node, mut network) = bootstrap(config);
        loop {
            let timeout = node.step(&mut network);
            if node.blockchain().len() > 2 {
                break;
            }
            network.await_events(timeout);
        }
        for handle in node_handles {
            handle.join().expect("node panicked");
        }
    }
}

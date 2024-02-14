// Remove this when it's not a WIP
#![allow(dead_code)]

//! Routines for bootstrapping a blockchat network of a given configuration.

use std::net::SocketAddr;

use crate::backend::Message;
use crate::network::broadcast::Broadcaster;
use crate::network::discovery::{discover_peers, bootstrap_helper};
use crate::wallet::Wallet;

pub struct BootstrapConfig {
    /// Whether this node is responsible for running the bootstrap helper
    bootstrap_leader: bool,
    /// The capacity per block.
    capacity: usize,
    // The number of expected nodes in the system.
    peers: usize,
    /// The socket address of the bootstrap helper.
    bootstrap_addr: SocketAddr,
    /// The socket address this node should listen to.
    listen_addr: SocketAddr,
    /// The wallet of this node.
    wallet: Wallet,
}

fn bootstrap(config: BootstrapConfig) {
    if config.bootstrap_leader {
        std::thread::spawn(move || bootstrap_helper(config.bootstrap_addr, config.peers));
    }

    let (my_index, peer_addrs, _peer_public_keys) = discover_peers(config.bootstrap_addr, config.listen_addr, config.wallet.public_key);

    let _broadcaster = Broadcaster::<Message>::new(&peer_addrs, my_index);

    // Instantiate node with the broadcaster network and given wallet.
    todo!();
}

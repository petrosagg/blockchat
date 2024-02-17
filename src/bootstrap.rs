// Remove this when it's not a WIP
#![allow(dead_code)]

//! Routines for bootstrapping a blockchat network of a given configuration.

use std::net::{IpAddr, SocketAddr, TcpListener};

use serde::{Deserialize, Serialize};

use crate::backend::Message;
use crate::crypto::PublicKey;
use crate::network::broadcast::Broadcaster;
use crate::network::discovery::{bootstrap_helper, discover_peers};
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
    listen_ip: IpAddr,
    /// The wallet of this node.
    wallet: Wallet,
}

/// The peer info exchanged during discovery.
#[derive(Serialize, Deserialize)]
struct PeerInfo {
    /// The socket address the peer will listen on.
    listen_addr: SocketAddr,
    /// The public key of this peer.
    public_key: PublicKey,
}

fn bootstrap(config: BootstrapConfig) {
    if config.bootstrap_leader {
        std::thread::spawn(move || {
            bootstrap_helper::<PeerInfo>(config.bootstrap_addr, config.peers)
        });
    }

    let listener = TcpListener::bind((config.listen_ip, 0)).unwrap();

    let peer_info = PeerInfo {
        listen_addr: listener.local_addr().unwrap(),
        public_key: config.wallet.public_key,
    };
    let (my_index, peer_infos) = discover_peers(config.bootstrap_addr, peer_info);

    let peer_addrs: Vec<_> = peer_infos.iter().map(|info| info.listen_addr).collect();
    let _broadcaster = Broadcaster::<Message>::new(listener, &peer_addrs, my_index);

    // Instantiate node with the broadcaster network and given wallet.
    todo!();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_boostrap() {}
}

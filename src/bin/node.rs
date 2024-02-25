use std::net::{IpAddr, SocketAddr};

use blockchat::network::Network;
use blockchat::{
    bootstrap::{self, BootstrapConfig},
    crypto,
};
use clap::Parser;
use log::LevelFilter;

/// A node for the BlockChat blockchain network.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Whether this node should lead the bootstrap phase. Exactly one of the participating nodes
    /// must set this flag.
    #[arg(long)]
    bootstrap_leader: bool,
    /// The number of expected peers in the network.
    #[arg(long)]
    peers: usize,
    /// The address of the bootstrap server.
    #[arg(long, default_value = "127.0.0.1:7000")]
    bootstrap_addr: SocketAddr,
    /// The IP address to bind to.
    #[arg(long, default_value = "127.0.0.1")]
    listen_ip: IpAddr,
    /// The maximum block capacity.
    #[arg(long, default_value = "5")]
    block_capacity: usize,
}

fn main() -> std::io::Result<()> {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    let args = Args::parse();

    let (private_key, public_key) = crypto::generate_keypair();
    let config = BootstrapConfig {
        bootstrap_leader: args.bootstrap_leader,
        capacity: args.block_capacity,
        peers: args.peers,
        bootstrap_addr: args.bootstrap_addr,
        listen_ip: args.listen_ip,
        public_key,
        private_key,
    };

    let (mut node, mut network) = bootstrap::bootstrap(config);

    loop {
        let timeout = node.step(&mut network);
        network.await_events(timeout);
    }

    Ok(())
}

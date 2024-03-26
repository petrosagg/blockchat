use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use clap::Parser;

use blockchat::bootstrap::{self, BootstrapConfig};
use blockchat::crypto::{self, Address};
use blockchat::network::Network;

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
    /// The path of the benchmark data. Should contain the trans<id>.txt files.
    #[arg(long)]
    bench_data: PathBuf,
    /// The stake amount this node should use.
    #[arg(long, default_value = "10")]
    stake: u64,
}

fn main() {
    tracing_subscriber::fmt::init();

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

    let (mut node, mut network, my_index, peers) = bootstrap::bootstrap(config);

    let data_path = args.bench_data.join(format!("trans{my_index}.txt"));
    let bench_data = std::fs::read_to_string(data_path).unwrap();
    let mut messages = vec![];
    for line in bench_data.lines() {
        let peer_id = line[2..3].parse::<usize>().unwrap();
        if let Some(info) = peers.get(peer_id) {
            let addr = Address::from_public_key(&info.public_key);
            messages.push((addr, line[4..].to_owned()));
        }
    }

    // Run the node until we get the initial funds
    while node.wallet().available_funds() == 0 {
        node.step(&mut network);
        network.await_events(Some(Duration::from_millis(15)));
    }

    // Run the node until we get the initial funds
    let tx = node.wallet().create_stake_tx(args.stake);
    let signed_tx = node.sign_transaction(tx);
    node.wallet_mut().apply_tx(signed_tx.clone()).unwrap();
    node.broadcast_transaction(signed_tx.clone());

    let start = Instant::now();

    for (recipient, message) in messages {
        println!(
            "Sending message to {recipient}. Available funds: {}",
            node.wallet().available_funds()
        );
        let tx = node.wallet().create_message_tx(recipient, message);
        if node.wallet().available_funds() < tx.cost() {
            println!("Waiting for more funds");
            while node.wallet().available_funds() < tx.cost() {
                node.step(&mut network);
                network.await_events(Some(Duration::from_millis(15)));
            }
            println!("Got more funds");
        }
        let signed_tx = node.sign_transaction(tx);
        node.wallet_mut().apply_tx(signed_tx.clone()).unwrap();
        node.broadcast_transaction(signed_tx.clone());
    }

    while node.has_pending_transactions() {
        node.step(&mut network);
        network.await_events(Some(Duration::from_millis(15)));
    }
    println!("Time to settle: {:?}", start.elapsed());
}

use std::collections::HashMap;
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
        // Give more initial funds so that the network can run through the required number of
        // transactions.
        genesis_funds_per_node: 10_000,
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

    // Run the node until we get the genesis funds
    println!("Waiting for funds");
    while node.wallet().available_funds() == 0 {
        node.step(&mut network);
        network.await_events(Some(Duration::from_millis(15)));
    }

    // Set up staking of this node
    println!("Setting up stake");
    let tx = node.wallet().create_stake_tx(args.stake);
    let signed_tx = node.sign_transaction(tx);
    node.wallet_mut().apply_tx(signed_tx.clone()).unwrap();
    node.broadcast_transaction(signed_tx.clone());

    while node.total_transactions() != (2 * args.peers) {
        node.step(&mut network);
        network.await_events(Some(Duration::from_millis(15)));
    }

    let start = Instant::now();

    for (recipient, message) in messages {
        let tx = node.wallet().create_message_tx(recipient, message);
        let signed_tx = node.sign_transaction(tx);
        node.wallet_mut().apply_tx(signed_tx.clone()).unwrap();
        node.broadcast_transaction(signed_tx.clone());
    }

    while node.total_transactions() != (2 * args.peers + 240) {
        node.step(&mut network);
        network.await_events(Some(Duration::from_millis(15)));
    }

    let mut block_counts = HashMap::new();
    for block in node.blockchain() {
        *block_counts.entry(block.public_key.clone()).or_insert(0) += 1;
    }
    for (i, (_, count)) in block_counts.into_iter().enumerate() {
        println!("Node {i} minted {count} blocks");
    }

    let total_blocks = node.blockchain().len() as f64;
    let total_txs: usize = node
        .blockchain()
        .iter()
        .map(|block| block.data.transactions.len())
        .sum();
    let total_txs = total_txs as f64;
    let total_time = (start.elapsed().as_millis() as f64) / 1000.0;
    let throughput = total_txs / total_time;
    let block_time = total_blocks / total_time;

    println!("Throughput {throughput}tx/s");
    println!("Block time {block_time}blocks/s");
}

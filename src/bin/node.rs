use std::{
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
};

use axum::{extract::State, routing::get, Json, Router};
use clap::Parser;
use tokio::net::TcpListener;

use blockchat::{
    bootstrap::{self, BootstrapConfig},
    crypto,
};
use blockchat::{
    crypto::Signed,
    network::Network,
    node::{Block, Node},
};

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

#[tokio::main]
async fn main() {
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

    let (node, mut network) = bootstrap::bootstrap(config);

    let shared_node = Arc::new(Mutex::new(node));
    // Start a thread that will run the node
    let node = Arc::clone(&shared_node);
    std::thread::spawn(move || loop {
        let timeout = { node.lock().unwrap().step(&mut network) };
        network.await_events(timeout);
    });

    let app = Router::new()
        .route("/block", get(get_block))
        .with_state(shared_node);

    let listener = TcpListener::bind((args.listen_ip, 0)).await.unwrap();

    log::info!(
        "Node HTTP API listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn get_block(State(node): State<Arc<Mutex<Node>>>) -> Json<Signed<Block>> {
    Json(node.lock().unwrap().blockchain().last().cloned().unwrap())
}

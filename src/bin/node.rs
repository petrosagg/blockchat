use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use blockchat::{
    bootstrap::{self, BootstrapConfig},
    crypto::{self, Address},
    wallet::{Transaction, Wallet},
};
use blockchat::{
    cli::client::{CreateTransactionRequest, SetStakeRequest},
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
    /// The base port for the HTTP API. Each node will start its HTTP server on
    /// `localhost:(api_base_port + node_index)`.
    #[arg(long, default_value = "10000")]
    api_base_port: u16,
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

    let (node, mut network, my_index) = bootstrap::bootstrap(config);

    let shared_node = Arc::new(Mutex::new(node));
    // Start a thread that will run the node
    let node = Arc::clone(&shared_node);
    std::thread::spawn(move || loop {
        let _ = { node.lock().unwrap().step(&mut network) };
        network.await_events(Some(Duration::from_millis(15)));
    });

    let app = Router::new()
        .route("/block", get(get_block))
        .route("/balance", get(get_balance))
        .route("/stake", post(set_stake))
        .route("/transaction", post(create_transaction))
        .with_state(shared_node);

    let api_port = args.api_base_port + u16::try_from(my_index).unwrap();
    let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), api_port))
        .await
        .unwrap();

    log::info!(
        "Node HTTP API listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn get_block(State(node): State<Arc<Mutex<Node>>>) -> Json<Signed<Block>> {
    Json(node.lock().unwrap().blockchain().last().cloned().unwrap())
}

async fn get_balance(State(node): State<Arc<Mutex<Node>>>) -> Json<Wallet> {
    Json(node.lock().unwrap().wallet().clone())
}

async fn create_transaction(
    State(node): State<Arc<Mutex<Node>>>,
    Json(req): Json<CreateTransactionRequest>,
) -> (StatusCode, Json<Signed<Transaction>>) {
    let mut node = node.lock().unwrap();
    let wallet = node.wallet();
    let tx = match req {
        CreateTransactionRequest::Coin { recipient, amount } => {
            wallet.create_coin_tx(recipient, amount)
        }
        CreateTransactionRequest::Message { recipient, message } => {
            wallet.create_message_tx(recipient, message)
        }
    };
    let signed_tx = node.sign_transaction(tx);
    node.broadcast_transaction(signed_tx.clone());
    (StatusCode::CREATED, Json(signed_tx))
}

async fn set_stake(
    State(node): State<Arc<Mutex<Node>>>,
    Json(req): Json<SetStakeRequest>,
) -> (StatusCode, Json<Signed<Transaction>>) {
    let mut node = node.lock().unwrap();
    let tx = node.wallet().create_stake_tx(req.amount);
    let signed_tx = node.sign_transaction(tx);
    node.broadcast_transaction(signed_tx.clone());
    (StatusCode::CREATED, Json(signed_tx))
}

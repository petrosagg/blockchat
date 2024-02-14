use std::net::SocketAddr;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(long)]
    bootstrap_addr: SocketAddr,

    #[arg(long)]
    bootstrap: bool,

    #[arg(long)]
    network_size: usize,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("received arguments {args:?}");
    Ok(())
}

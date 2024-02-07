use std::{
    io::{BufRead, BufReader, Write},
    net::{Ipv4Addr, TcpListener, TcpStream},
    time::Duration,
};

use clap::Parser;
use serde::{Deserialize, Serialize};

pub mod broadcast;
pub mod bootstrap;
pub mod crypto;
pub mod backend;
pub mod error;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(long)]
    bootstrap_host: Ipv4Addr,

    #[arg(long)]
    bootstrap_port: u16,

    #[arg(long)]
    bootstrap: bool,

    #[arg(long)]
    network_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
enum NodeCommand {
    Bootstrap,
}

#[derive(Debug, Serialize, Deserialize)]
enum NodeResponse {
    BootstrapSuccess(usize),
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("received arguments {args:?}");
    if args.bootstrap {
        let mut node_id = 1;

        let listener = TcpListener::bind((args.bootstrap_host, args.bootstrap_port))?;
        // Should start listening on bootstrap host/port
        for stream in listener.incoming() {
            let mut stream = BufReader::new(stream?);
            let peer_addr = stream.get_ref().peer_addr()?;

            let mut command_bytes = String::new();
            if stream.read_line(&mut command_bytes).unwrap() > 0 {
                let command: NodeCommand = serde_json::from_str(&command_bytes)?;

                let response = NodeResponse::BootstrapSuccess(node_id);
                node_id += 1;
                let bytes = serde_json::to_writer(stream.get_mut(), &response);
                stream.get_mut().write(&[b'\n'])?;
                stream.get_mut().flush()?;
            }
            std::thread::sleep(Duration::from_secs(60));
        }
    } else {
        // Should connect to bootstrap host/port
        let mut stream = TcpStream::connect((args.bootstrap_host, args.bootstrap_port))?;
        let mut stream = BufReader::new(stream);

        let command = NodeCommand::Bootstrap;
        let bytes = serde_json::to_writer(stream.get_mut(), &command);
        stream.get_mut().write(&[b'\n'])?;
        stream.get_mut().flush()?;

        let mut response_bytes = String::new();
        if stream.read_line(&mut response_bytes).unwrap() > 0 {
            let response: NodeResponse = serde_json::from_str(&response_bytes)?;
            println!("Bootstrap successful, received response {response:?}")
        }

        std::thread::sleep(Duration::from_secs(60));
    }
    Ok(())
}

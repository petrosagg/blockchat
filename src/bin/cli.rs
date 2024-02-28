use clap::Parser;
use reqwest::Url;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

use blockchat::cli::client::BlockchatClient;
use blockchat::cli::command::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The URL of the RPC node.
    #[arg(long)]
    rpc_url: Url,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("Using RPC at {}", args.rpc_url);
    let client = BlockchatClient::new(args.rpc_url);

    let mut rl = DefaultEditor::new()?;
    loop {
        let readline = rl.readline("blockchat> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                match line.parse::<Command>() {
                    Ok(cmd) => {
                        cmd.run(client.clone()).await;
                    }
                    Err(err) => println!("Error: {err:?}"),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

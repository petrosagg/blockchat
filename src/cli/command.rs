//! The various commands supported by the CLI

use std::str::FromStr;

use crate::crypto::PublicKey;

pub enum Command {
    NewTransaction(NewTransactionCommand),
    NewMessage(NewMessageCommand),
    Stake(StakeCommand),
    ViewLastBlockCommand,
    ShowBalanceCommand,
    HelpCommand,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(cmd: &str) -> Result<Self, Self::Err> {
        let cmd = cmd.trim();
        Ok(match cmd {
            "view" => Command::ViewLastBlockCommand,
            "balance" => Command::ShowBalanceCommand,
            "help" => Command::HelpCommand,
            cmd if cmd.starts_with("t ") => Command::NewTransaction(cmd.parse()?),
            cmd if cmd.starts_with("m ") => Command::NewMessage(cmd.parse()?),
            cmd if cmd.starts_with("stake ") => Command::Stake(cmd.parse()?),
            cmd => return Err(format!("invalid command: {cmd}")),
        })
    }
}

pub struct NewTransactionCommand {
    recipient: PublicKey,
    amount: String,
}

impl FromStr for NewTransactionCommand {
    type Err = String;

    fn from_str(_cmd: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

pub struct NewMessageCommand {
    recipient: PublicKey,
    message: String,
}

impl FromStr for NewMessageCommand {
    type Err = String;

    fn from_str(_cmd: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

pub struct StakeCommand {
    amount: u64,
}

impl FromStr for StakeCommand {
    type Err = String;

    fn from_str(_cmd: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}

//! The various commands supported by the CLI

use std::str::FromStr;

use crate::crypto::PublicKey;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct NewTransactionCommand {
    pub recipient: PublicKey,
    pub amount: String,
}

impl FromStr for NewTransactionCommand {
    type Err = String;

    fn from_str(_cmd: &str) -> Result<Self, Self::Err> {
        // TODO parse cmd which is in the form of "t <public_key> <amount>"
        todo!()
    }
}

#[derive(Debug)]
pub struct NewMessageCommand {
    pub recipient: PublicKey,
    pub message: String,
}

impl FromStr for NewMessageCommand {
    type Err = String;

    fn from_str(_cmd: &str) -> Result<Self, Self::Err> {
        // TODO parse cmd which is in the form of "t <public_key> <message>"
        todo!()
    }
}

#[derive(Debug)]
pub struct StakeCommand {
    pub amount: u64,
}

impl FromStr for StakeCommand {
    type Err = String;

    fn from_str(_cmd: &str) -> Result<Self, Self::Err> {
        // TODO parse cmd which is in the form of "stake <amount>"
        todo!()
    }
}

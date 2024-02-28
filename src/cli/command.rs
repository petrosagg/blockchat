//! The various commands supported by the CLI

use std::str::FromStr;

use crate::crypto::Address;

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

impl Command {
    pub fn run(&self) {
        match self {
            Command::NewTransaction(tx) => todo!(),
            Command::NewMessage(tx) => todo!(),
            Command::Stake(tx) => todo!(),
            Command::ViewLastBlockCommand => todo!(),
            Command::ShowBalanceCommand => todo!(),
            Command::HelpCommand => todo!(),
        }
    }
}

#[derive(Debug)]
pub struct NewTransactionCommand {
    pub recipient: Address,
    pub amount: u64,
}

impl FromStr for NewTransactionCommand {
    type Err = String;

    fn from_str(_cmd: &str) -> Result<Self, Self::Err> {
        let mut parts = _cmd.split_whitespace();

        assert_eq!(parts.next(), Some("t"));

        let recipient = match parts.next() {
            Some(r) => r.parse::<Address>()?,
            _ => return Err("No recipient address provided.".into()),
        };

        let amount = match parts.next() {
            Some(a) => a
                .parse::<u64>()
                .map_err(|_| "Could not parse amount.".to_owned())?,
            None => return Err("No amount provided.".into()),
        };

        Ok(NewTransactionCommand { recipient, amount })
    }
}

#[derive(Debug)]
pub struct NewMessageCommand {
    pub recipient: Address,
    pub message: String,
}

impl FromStr for NewMessageCommand {
    type Err = String;

    fn from_str(_cmd: &str) -> Result<Self, Self::Err> {
        let mut parts = _cmd.split_whitespace();

        assert_eq!(parts.next(), Some("m"));

        let recipient = match parts.next() {
            Some(r) => r.parse::<Address>()?,
            _ => return Err("No recipient address provided.".into()),
        };

        let message = match parts.next() {
            Some(a) => a.to_owned(),
            None => return Err("No message provided.".into()),
        };

        Ok(NewMessageCommand { recipient, message })
    }
}

#[derive(Debug)]
pub struct StakeCommand {
    pub amount: u64,
}

impl FromStr for StakeCommand {
    type Err = String;

    fn from_str(_cmd: &str) -> Result<Self, Self::Err> {
        let mut parts = _cmd.split_whitespace();

        assert_eq!(parts.next(), Some("stake"));

        let amount = match parts.next() {
            Some(a) => a
                .parse::<u64>()
                .map_err(|_| "Could not parse amount.".to_owned())?,
            None => return Err("No amount provided.".into()),
        };

        Ok(StakeCommand { amount })
    }
}

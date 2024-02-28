use reqwest::Url;

use crate::crypto::Address;

pub struct Client {
    rpc_url: Url,
}

type Err = String;

impl Client {
    pub fn new(rpc_url: Url) -> Self {
        Client { rpc_url }
    }
    // TODO: Make recipient Address type
    pub fn send_transaction(&self, recipient: Address, amount: u64) -> Result<(Address, u64), Err> {
        // TODO: Call API to send coin transaction
        todo!();
    }

    // TODO: Make recipient Address type
    pub fn send_message(
        &self,
        recipient: Address,
        message: String,
    ) -> Result<(Address, String), Err> {
        // TODO: Call API to send message transaction
        todo!();
    }

    pub fn stake(&self, amount: u64) -> Result<(u64), Err> {
        // TODO: Call API to send stake transaction
        todo!();
    }
}

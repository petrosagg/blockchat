use reqwest::{Url, Client};

use crate::{crypto::{Address, Signed}, node::Block, wallet::Wallet};

#[derive(Clone)]
pub struct BlockchatClient {
    rpc_url: Url,
    client:  Client
}

type Err = String;

impl BlockchatClient {
    pub fn new(rpc_url: Url) -> Self {
        BlockchatClient {
          rpc_url,
          client: Client::new()
        }
    }

    pub async fn get_balance(&self) -> Result<Wallet, Err> {
        let request = self.client.get(self.rpc_url.join("balance").unwrap());
        let response = request.send().await.unwrap();
        let wallet = response.json::<Wallet>().await.unwrap();

        Ok(wallet)
    }

    pub async fn get_last_block(&self) -> Result<Signed<Block>, Err> {
        let request = self.client.get(self.rpc_url.join("block").unwrap());
        let response = request.send().await.unwrap();
        let last_block = response.json::<Signed<Block>>().await.unwrap();

        Ok(last_block)
    }

    pub fn send_transaction(&self, recipient: Address, amount: u64) -> Result<(Address, u64), Err> {
        // TODO: Call API to send coin transaction
        todo!();
    }

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

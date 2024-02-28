use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{Address, Signed},
    node::Block,
    wallet::{Transaction, Wallet},
};

#[derive(Clone)]
pub struct BlockchatClient {
    rpc_url: Url,
    client: Client,
}

type Err = String;

#[derive(Serialize, Deserialize)]
pub struct SetStakeRequest {
    pub amount: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateTransactionRequest {
    Coin { recipient: Address, amount: u64 },
    Message { recipient: Address, message: String },
}

impl BlockchatClient {
    pub fn new(rpc_url: Url) -> Self {
        BlockchatClient {
            rpc_url,
            client: Client::new(),
        }
    }

    pub async fn get_balance(&self) -> Result<Wallet, Err> {
        let request = self.client.get(self.rpc_url.join("balance").unwrap());
        let response = request.send().await.unwrap();
        let wallet = response.json::<Wallet>().await.unwrap();

        Ok(wallet)
    }

    pub async fn get_last_block(&self) -> Result<Signed<Block>, Err> {
        let url = self.rpc_url.join("block").unwrap();
        let request = self.client.get(url);
        let response = request.send().await.unwrap();
        let last_block = response.json().await.unwrap();

        Ok(last_block)
    }

    pub async fn send_transaction(
        &self,
        recipient: Address,
        amount: u64,
    ) -> Result<Signed<Transaction>, Err> {
        let url = self.rpc_url.join("transaction").unwrap();
        let request = self
            .client
            .post(url)
            .json(&CreateTransactionRequest::Coin { recipient, amount });
        let response = request.send().await.unwrap();
        let tx = response.json().await.unwrap();

        Ok(tx)
    }

    pub async fn send_message(
        &self,
        recipient: Address,
        message: String,
    ) -> Result<Signed<Transaction>, Err> {
        let url = self.rpc_url.join("transaction").unwrap();
        let request = self
            .client
            .post(url)
            .json(&CreateTransactionRequest::Message { recipient, message });
        let response = request.send().await.unwrap();
        let tx = response.json().await.unwrap();

        Ok(tx)
    }

    pub async fn stake(&self, amount: u64) -> Result<Signed<Transaction>, Err> {
        let url = self.rpc_url.join("stake").unwrap();
        let request = self.client.post(url).json(&SetStakeRequest { amount });
        let response = request.send().await.unwrap();

        let stake_tx = response.json().await.unwrap();
        Ok(stake_tx)
    }
}

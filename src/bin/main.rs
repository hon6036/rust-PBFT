mod client;
extern crate log4rs;
use reqwest::Client;
use std::fs;
use std::error::Error;
use std::path::Path;
use serde::{Serialize,Deserialize};
#[derive(Deserialize)]
pub struct ClientConfig {
    transaction_number: i32,
    http_address: String
}
#[derive(Serialize)]
pub struct Transaction {
    sender: String,
    receiver: String,
    balance: i32
}

fn load_config() -> Result<ClientConfig, Box<dyn Error>> {
    let config_path = Path::new("././client_config.toml");
    let config_str = fs::read_to_string(config_path)?;
    let config: ClientConfig = toml::from_str(&config_str)?;
    Ok(config)
}

fn make_transaction() -> Transaction {
    let transaction = Transaction {
        sender: "1".to_string(),
        receiver: "2".to_string(),
        balance: 12,
    };
    transaction
}

async fn send_transaction(client:Client, transaction:String, address: String) -> Result<reqwest::Response, reqwest::Error> {
    let res = client.post(address)
        .header("Content-Type", "application/json")
        .body(transaction)
        .send()
        .await;
    res
}
#[tokio::main]
async fn main() {
    let log_file = Path::new("./log.yml");
    log4rs::init_file(log_file, Default::default()).unwrap();
    let config = load_config().unwrap();
    let client = reqwest::Client::new();
    let http_address = config.http_address;
    let sender = Client::new();
    for _i in 0..config.transaction_number{
        let transaction = make_transaction();
        let transaction = serde_json::to_string(&transaction).unwrap();
        let _ = send_transaction(client.clone(), transaction, http_address.clone()).await;
    }
}
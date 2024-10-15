mod client;
extern crate log4rs;
use log::info;
use rand::Rng;
use reqwest::Client;
use std::{env, fs};
use std::error::Error;
use std::path::Path;
use serde::{Serialize,Deserialize};
use std::fs::{OpenOptions,File};
use std::io::{BufRead, BufReader, Write};

#[derive(Deserialize)]
pub struct ClientConfig {
    transaction_number: i32,
    http_address: String
}
#[derive(Serialize, Debug)]
pub struct Transaction {
    from: String,
    to: String,
    balance: i32
}

fn load_config() -> Result<ClientConfig, Box<dyn Error>> {
    let config_path = Path::new("././client_config.toml");
    let config_str = fs::read_to_string(config_path)?;
    let config: ClientConfig = toml::from_str(&config_str)?;
    Ok(config)
}

fn make_transaction() -> Transaction {
    let file_path = "address.txt";
    let mut file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);
    let line_count = reader.lines().count();

    let mut file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let from_number = rand::thread_rng().gen_range(0..=line_count);
    let from = reader.lines().nth(from_number).unwrap().unwrap();
    
    let mut file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let to_number = rand::thread_rng().gen_range(0..=line_count);
    let to = reader.lines().nth(to_number).unwrap().unwrap();

    let balance = rand::thread_rng().gen_range(0..=10000);
    let transaction = Transaction {
        from,
        to,
        balance
    };
    transaction
}

fn make_clients_address() {
    let file_path = "address.txt";
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .expect("Cannot open file");
    for _i in 0..10000 {
        let client = client::Client::new();
        writeln!(file, "{}", client.address).expect("Fail to write file");
    }
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
    // make_clients_address()
    for _i in 0..config.transaction_number{
        let transaction = make_transaction();
        let transaction = serde_json::to_string(&transaction).unwrap();
        let _ = send_transaction(client.clone(), transaction, http_address.clone()).await;
    }
}
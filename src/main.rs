extern crate log4rs;
use std::sync::Arc;
use std::thread;
mod consensus;
mod replica;
mod message;
mod types;
mod transport;
mod socket;
mod http;
mod mempool;
mod crypto;
use serde::Deserialize;
use std::fs;
use std::error::Error;
use std::path::Path;
#[derive(Deserialize)]
pub struct ServerConfig {
    replica_number: i32,
    consensus: String,
    batch_size: usize
}

pub fn load_config() -> Result<ServerConfig, Box<dyn Error>> {
    let config_path = Path::new("./server_config.toml");
    let config_str = fs::read_to_string(config_path)?;
    let config: ServerConfig = toml::from_str(&config_str)?;
    Ok(config)
}

fn main() {
    let mut handles = vec![];
    let log_file = Path::new("./log.yml");
    log4rs::init_file(log_file, Default::default()).unwrap();
    let config = load_config().unwrap();
    let consensus = Arc::new(config.consensus);
    for i in 0..config.replica_number {
        let consensus_clone = consensus.clone();
        let handle = thread::spawn(move|| {
            let replica = replica::Replica::new(i, consensus_clone.to_string());
            replica.start();
        });
        handles.push(handle)
    }

    for handle in handles{
        handle.join().unwrap()
    }
        
}

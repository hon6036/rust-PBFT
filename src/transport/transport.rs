use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use log::{info,error};
use crate::consensus::Consensus;
use crate::replica::*;
use crate::message::Message;

pub struct Transport {
    connection: TcpListener,
}

impl Transport {
    
    pub fn new(id:i32) -> Transport {
        let connection = Self::make_listener(id);
        
        Transport{connection}
    }
    
    pub fn connection(&self) -> &TcpListener {
        &self.connection
    }
    
    pub fn make_listener(id:i32) -> TcpListener {
        let address = format!("127.0.0.1:{}", 20000 + id);
        TcpListener::bind(address).expect("Failed bind address")
    }
    
    pub fn handle_connection(consensus: Arc<Mutex<Consensus>>, id: Arc<String>, mut stream:TcpStream) {
        let mut buffer = [0 as u8;200];
        loop {
            match stream.read(&mut buffer) {
                Ok(_) => {
                    match bincode::deserialize(&buffer) {
                        Ok(parsed_message) => {
                            info!("received data {:?}", parsed_message);
                            match parsed_message {
                                Message::PrePrePare(data) => Replica::handle_preprepare_message(consensus.clone(),id.clone(),data),
                                Message::PrePare(data) => Replica::handle_prepare_message(consensus.clone(), id.clone(),data),
                                Message::COMMIT(data) => Replica::handle_commit_message(consensus.clone(), id.clone(),data)
                            }
                        }
                        Err(err) => {
                            error!("Failed to parsing data {}", err)
                        }
                    }
                },
                Err(err) => {
                    error!("Failed to read data {}", err);
                    
                }
            } {}
        }
    }
}

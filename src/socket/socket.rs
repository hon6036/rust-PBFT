use std::collections::HashMap;
use std::io::prelude::*;
use std::net::TcpStream;
use bincode::serialize;
use log::info;
use crate::load_config;
use crate::types::*;
use crate::message::Message;

pub struct Socket{
    id: Identity,
    nodes: HashMap<Identity, TcpStream>
}

impl Socket {
    
    pub fn new(id:Identity) -> Socket {
        let nodes = HashMap::new();
        Socket{id, nodes}
    }

    pub fn make_stream(id:i32) -> TcpStream {
        let address = format!("127.0.0.1:{}", 20000 + id);
        TcpStream::connect(address).expect("Failed make stream")  
    }
    
    pub fn send(&mut self, to:Identity, byte_message:&[u8]) {
        let to_i32 : i32 = to.parse().expect("Not a vaild number");
        match self.nodes.get(&to) {
            Some(mut stream) => {
                info!("send");
                let _ = stream.write_all(byte_message);
            },
            None => {
                let mut stream:TcpStream = Self::make_stream(to_i32);
                let _ = stream.write_all(byte_message);
                self.nodes.insert(to, stream);
            }
        }
    }

    pub fn broadcast(&mut self, message:Message) {
        let byte_vector: Vec<u8> = serialize(&message).expect("Failed to serialize message");
        let byte_message: &[u8] = &byte_vector;
        let config = load_config().unwrap();
        for i in 0..config.replica_number {
            if i.to_string() == self.id {
                continue
            }
            self.send(i.to_string(), byte_message)
        }
    }
}
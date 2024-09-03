use log::info;

use crate::{types::Identity, message::*, socket::*};
use std::sync::{Arc, Mutex};

pub struct PBFT {
    id: Identity,
    socket: Arc<Mutex<socket::Socket>>

}


impl PBFT{
    pub fn new(id: i32) -> PBFT{
        let id = id.to_string();
        let socket = Arc::new(Mutex::new(Socket::new(id.clone())));
        PBFT{id, socket}
    }

    pub fn make_block(&self) {
        todo!()
    }

    pub fn process_preprepare(&self) {
        info!(" start processing block");
        let message = PrePrePare{
            view: 1,
            block_height: 1,
            id: "2".to_string(),
        };
        let socket = self.socket.clone();
        let mut socket = socket.lock().unwrap();
        socket.broadcast(message::Message::PrePrePare(message))
    }

    pub fn process_prepare(&self) {
        todo!()
    }

    pub fn process_commit(&self) {

    }
}
use log::{debug, info};
use ring::signature::{EcdsaKeyPair, KeyPair, UnparsedPublicKey, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING};

use crate::{blockchain::{block, BlockWithoutSignature}, crypto::{self, *}, crypto::{*}, load_config, mempool::*, message::{self, *}, socket::*, types::{self, Identity}};
use std::{collections::HashMap, sync::{Arc, Mutex}};

pub struct PBFT {
    id: Identity,
    socket: Arc<Mutex<socket::Socket>>,
    publickeys: HashMap<Identity,Vec<u8>>,
    key_pair: EcdsaKeyPair
}


impl PBFT{
    pub fn new(id: i32, key_pair:EcdsaKeyPair) -> PBFT{
        let id = id.to_string();
        let socket = Arc::new(Mutex::new(Socket::new(id.clone())));
        let publickeys = HashMap::new();
        PBFT{id, socket, publickeys, key_pair}
    }

    pub fn exchange_publickey(&self){
        let publickey = self.key_pair.public_key().as_ref();
        let socket = self.socket.clone();
        let mut socket = socket.lock().unwrap();
        let publickey = message::PublicKey {
            id: self.id.to_string(),
            publickey: publickey.to_vec()
        };
        socket.broadcast(message::message::Message::PublicKey(publickey))
    }

    pub fn store_publickey(&mut self, id:types::Identity,publickey: Vec<u8>) {
        self.publickeys.insert(id,publickey);

    }

    pub fn make_block(&self, mempool:Arc<Mutex<MemPool>>) {
        info!("make block start");
        let mut mempool = mempool.lock().unwrap();
        let config = load_config().unwrap();
        let batch_size = config.batch_size;
        let payload = mempool.payload(batch_size);
        let block_without_signature: BlockWithoutSignature = block::BlockWithoutSignature {
            payload,
            view: 1,
            block_height: 1,
            proposer: 1.to_string()
        };
        let block_id = make_block_id(&block_without_signature);
        let signature = make_block_signature(&self.key_pair, &block_without_signature);
        let payload = block_without_signature.payload;
        let id = &self.id;
        let block = block::Block{
            block_id,
            payload,
            signature,
            block_height: 1,
            view: 1,
            proposer: id.to_string()
        };
        let socket = self.socket.clone();
        let mut socket = socket.lock().unwrap();
        let preprepare_message = message::PrePrePare {
            block
        };
        socket.broadcast(message::Message::PrePrePare(preprepare_message))
    }


    pub fn process_preprepare(&self,  message: PrePrePare) {
        info!("start processing block");
        let proposer_publickey = self.publickeys.get(&message.block.proposer).unwrap();
        if verify_signature(proposer_publickey.to_owned(), message::message::Message::PrePrePare(message)) {
            info!("block verify success")
        } else {
            info!("fail to verify")
        }
    }

    pub fn process_prepare(&self, message: PrePare) {
        info!("start process_prepare");
        let proposer_publickey = self.publickeys.get(&message.proposer).unwrap();
        if verify_signature(proposer_publickey.to_owned(), message::message::Message::PrePare(message)) {

        }
    }

    pub fn process_commit(&self, message: Commit) {
        info!("start process_commit");
        let proposer_publickey = self.publickeys.get(&message.proposer).unwrap();
        if verify_signature(proposer_publickey.to_owned(), message::message::Message::Commit(message)) {

        }
    }
}
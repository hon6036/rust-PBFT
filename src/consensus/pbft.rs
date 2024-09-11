use log::{error, info};
use ring::signature::{EcdsaKeyPair, KeyPair, UnparsedPublicKey, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING};
use tokio::{runtime::Runtime, sync::mpsc::Sender};

use crate::{blockchain::{self, block, BlockWithoutSignature}, crypto::{self, *}, crypto::{*}, load_config, mempool::*, message::{self, *}, quorum::{self, CommitQuroum, PrePareQuroum, Quorum}, socket::*, types::{self, Identity}};
use core::time;
use std::{collections::HashMap, sync::{Arc, Mutex}, thread::sleep};

pub struct PBFT {
    id: Identity,
    socket: Arc<Mutex<socket::Socket>>,
    publickeys: HashMap<Identity,Vec<u8>>,
    key_pair: EcdsaKeyPair,
    view_channel_tx: Sender<types::View>,
    current_prepare_quorum_certificate: types::View,
    prepare_quorum: Quorum,
    commit_quorum: Quorum,
    blockchain: blockchain::Blockchain,
    agreeing_block: blockchain::Block
}


impl PBFT{
    pub fn new(id: i32, key_pair:EcdsaKeyPair, view_channel_tx: Sender<types::View>, replica_number:i32) -> PBFT{
        let id = id.to_string();
        let socket = Arc::new(Mutex::new(Socket::new(id.clone())));
        let publickeys = HashMap::new();
        let current_prepare_quorum_certificate = 1;
        let prepare_quorum = PrePareQuroum::new(replica_number);
        let prepare_quorum = quorum::Quorum::PrePareQuroum(prepare_quorum);
        let commit_quorum = CommitQuroum::new(replica_number);
        let commit_quorum = quorum::Quorum::CommitQuroum(commit_quorum);
        let blockchain = blockchain::Blockchain::new(id.clone());
        let agreeing_block = block::Block::default();
        PBFT{id, socket, publickeys, key_pair, view_channel_tx, current_prepare_quorum_certificate, prepare_quorum, commit_quorum, blockchain, agreeing_block}
    }

    pub fn exchange_publickey(&self){
        let publickey = self.key_pair.public_key().as_ref();
        let socket = self.socket.clone();
        let mut socket = socket.lock().map_err(|poisoned| {
            error!("Socket is poisoned {:?}", poisoned);
        }).unwrap();
        let publickey = message::PublicKey {
            id: self.id.to_string(),
            publickey: publickey.to_vec()
        };
        socket.broadcast(message::Message::PublicKey(publickey))
    }

    pub fn store_publickey(&mut self, id:types::Identity,publickey: Vec<u8>) {
        self.publickeys.insert(id,publickey);

    }

    pub fn make_block(&mut self, mempool:Arc<Mutex<MemPool>>, view:types::View) {
        info!("make block start");
        let mut mempool = mempool.lock().map_err(|poisoned| {
            error!("mempool is poisoned {:?}", poisoned);
        }).unwrap();
        let config = load_config().unwrap();
        let batch_size = config.batch_size;
        let payload = mempool.payload(batch_size);
        if view == 1 {
            let parent_block_id = "0000000000000000000000000000000000000000000000000000000000000000";
            let block_height = 1;
            self.process_block(payload, view, block_height, parent_block_id.to_string())
        } else {
            let parent_block_id = self.agreeing_block.block_id.to_string();
            let block_height = self.agreeing_block.block_height;
            self.process_block(payload, view, block_height, parent_block_id)
        }
        
    }

    pub fn process_block(&mut self, payload:Vec<Transaction>, view:types::View, block_height:types::BlockHeight, parent_block_id:types::BlockID) {
        let block_without_signature: BlockWithoutSignature = block::BlockWithoutSignature {
            payload,
            view,
            block_height,
            proposer: 1.to_string(),
            parent_block_id: parent_block_id.clone()
        };
        let block_id = make_block_id(&block_without_signature);
        let serialized_block = serde_json::to_vec(&block_without_signature).map_err(|e| {
            error!("Serialized block without signature {:?}", e)
        }).unwrap();
        let signature = make_signature(&self.key_pair, &serialized_block);
        let payload = block_without_signature.payload;
        let id = &self.id;
        let block = block::Block{
            block_id,
            payload,
            signature,
            block_height,
            view,
            proposer: id.to_string(),
            parent_block_id

        };
        self.agreeing_block = block.clone();
        let socket = self.socket.clone();
        let mut socket = socket.lock().map_err(|poisoned| {
            error!("Socket is poisoned {:?}", poisoned);
        }).unwrap();
        let preprepare_message = message::PrePrePare {
            block
        };
        socket.broadcast(message::Message::PrePrePare(preprepare_message))
    }


    pub fn process_preprepare(&mut self,  preprepare_message: PrePrePare) {
        info!("start processing block");
        let proposer_publickey = self.publickeys.get(&preprepare_message.block.proposer).unwrap();
        let message = message::Message::PrePrePare(preprepare_message.clone());
        if verify_signature(proposer_publickey.to_owned(), message) {
            info!("success to verify block");
        } else {
            info!("fail to verify block");
            return
        }

        let view = preprepare_message.block.view.clone();
        let block_height = preprepare_message.block.block_height.clone();
        let socket = self.socket.clone();
        let mut socket = socket.lock().map_err(|poisoned| {
            error!("Socket is poisoned {:?}", poisoned);
        }).unwrap();
        let prepare_message_without_signature = message::PrePareWithoutSignature {
            view,
            block_height,
            proposer: self.id.clone()
        };

        let serialized_message = serde_json::to_vec(&prepare_message_without_signature).map_err(|e| {
            error!("Serialized prepare message without signature {:?}", e)
        }).unwrap();
        let signature = make_signature(&self.key_pair, &serialized_message);
        let prepare_message = message::PrePare {
            view,
            block_height,
            proposer: self.id.clone(),
            signature
        };
        Self::process_prepare(self, prepare_message.clone());
        socket.broadcast(message::Message::PrePare(prepare_message))
        // sleep(time::Duration::new(1, 0));
        // let rt = Runtime::new().unwrap();
        // rt.block_on(self.advance_view());
        

    }

    pub fn process_prepare(&mut self, prepare_message: PrePare) {
        info!("{:?} start process_prepare", self.id.clone());
        if self.id != prepare_message.proposer {
            let proposer_publickey = self.publickeys.get(&prepare_message.proposer).expect("fail to get proposer's publickey");
            let message = message::Message::PrePare(prepare_message.clone());
            if verify_signature(proposer_publickey.to_owned(), message) {
                info!("{:?} success to verify {:?} prepare message", self.id.clone(), prepare_message.proposer);
            } else {
                info!("{:?} fail to verify {:?} prepare message", self.id.clone(), prepare_message.proposer);
                return
            }
        }
        let prepare_quorum_certificate = Quorum::add(message::Message::PrePare(prepare_message), &mut self.prepare_quorum);
        if prepare_quorum_certificate.is_left(){
            info!("{:?} prepare quorum certification finished", self.id);
            let message = prepare_quorum_certificate.left().unwrap();
            self.process_messages(message)
        }
    }

    pub fn process_commit(&mut self, commit_message: Commit) {
        info!("{:?} start process_commit", self.id);
        if self.id != commit_message.proposer {
            let proposer_publickey = self.publickeys.get(&commit_message.proposer).expect("fail to get proposer's publickey");
            let message = message::Message::Commit(commit_message.clone());
            if verify_signature(proposer_publickey.to_owned(), message) {
                info!("{:?} success to verify {:?} commit message", self.id.clone(), commit_message.proposer);
            } else {
                info!("{:?} fail to verify {:?} commit message", self.id.clone(), commit_message.proposer);
                return
            }
        }
        let commit_quorum_certificate = Quorum::add(message::Message::Commit(commit_message), &mut self.commit_quorum);
        if commit_quorum_certificate.is_left(){
            info!("{:?} commit quorum certification finished", self.id);
            let message = commit_quorum_certificate.left().unwrap();
            self.process_messages(message)
        }
    }

    pub fn process_messages(&mut self, message:message::Message) {
        match message {
            Message::PrePare(prepare_message) => {
                info!("{:?} start process_messages", self.id);
                let view = prepare_message.view.clone();
                let block_height = prepare_message.block_height.clone();
                let socket = self.socket.clone();
                let mut socket = socket.lock().map_err(|poisoned| {
                    error!("Socket is poisoned {:?}", poisoned);
                }).unwrap();
                let commit_message_without_signature = message::CommitWithoutSignature {
                    view,
                    block_height,
                    proposer: self.id.clone()
                };
                info!("{:?} start serialize commit message without signature", self.id);
                
                let serialized_message = serde_json::to_vec(&commit_message_without_signature).map_err(|e| {
                    error!("Serialized commit message without signature {:?}", e)
                }).unwrap();
                info!("{:?} start make commit message signature", self.id);

                let signature = make_signature(&self.key_pair, &serialized_message);
                info!("{:?} finish make commit message signature", self.id);

                let commit_message = message::Commit {
                    view,
                    block_height,
                    proposer: self.id.clone(),
                    signature
                };
                self.process_commit(commit_message.clone());
                socket.broadcast(message::Message::Commit(commit_message))
            },
            Message::Commit(commit_message) => {
                let commit_message = commit_message;
                let block = &self.agreeing_block;
                self.blockchain.commit_block(block.clone());
                let rt = Runtime::new().map_err(|e| {
                    error!("Set new runtime at process messages which is commit {:?}", e)
                }).unwrap();
                rt.block_on(self.advance_view());
            },
            _ => todo!(),
        }
    }

    pub async fn advance_view(&mut self) {
        info!("{:?} advance_view start {:?}", self.id, self.current_prepare_quorum_certificate);
        self.current_prepare_quorum_certificate += 1;
        if self.id == 1.to_string() {
            if let Err(e) = self.view_channel_tx.send(self.current_prepare_quorum_certificate).await {
                info!("Failed to send view: {:?}", e.to_string())
            };
        }
    }
}
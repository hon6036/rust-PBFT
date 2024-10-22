use ecdsa::{Signature, SigningKey, VerifyingKey};
use k256::Secp256k1;
use log::{error, info};
use revm::{db::{CacheDB, DbAccount, EmptyDBTyped}, handler, inspector_handle_register, inspectors::NoOpInspector, primitives::{AccountInfo, Address, Bytecode, FixedBytes, HandlerCfg, TxEnv, TxKind, U256}, CacheState, Evm, EvmContext, Handler, InMemoryDB, JournaledState};
use tokio::{runtime::Runtime, sync::mpsc::Sender};
use std::{collections::HashSet, convert::Infallible, fs::File, hash::{DefaultHasher, Hash, Hasher}, str::FromStr};
use std::io::{BufRead, BufReader};
use crate::{blockchain::{self, block, BlockWithoutSignature}, crypto::{self, *}, crypto::{*}, load_config, mempool::*, message::{self, *}, quorum::{self, CommitQuroum, PrePareQuroum, Quorum}, socket::*, types::{self, Identity}};
use core::time;
use std::{collections::HashMap, sync::{Arc, Mutex}, thread::sleep};

pub struct PBFT {
    id: Identity,
    socket: Arc<Mutex<socket::Socket>>,
    verifyingkeys: HashMap<Identity,Vec<u8>>,
    signing_key: SigningKey<Secp256k1>,
    verifying_key: VerifyingKey<Secp256k1>,
    view_channel_tx: Sender<types::View>,
    current_prepare_quorum_certificate: types::View,
    prepare_quorum: Quorum,
    commit_quorum: Quorum,
    blockchain: blockchain::Blockchain,
    agreeing_block: blockchain::Block,
    journaled_state: JournaledState,
    in_memory_db: CacheDB<EmptyDBTyped<Infallible>>
}


impl PBFT{
    pub fn new(id: i32, signing_key:SigningKey<Secp256k1>, verifying_key:VerifyingKey<Secp256k1>, view_channel_tx: Sender<types::View>, replica_number:i32) -> PBFT{
        let id = id.to_string();
        let socket = Arc::new(Mutex::new(Socket::new(id.clone())));
        let verifyingkeys = HashMap::new();
        let current_prepare_quorum_certificate = 1;
        let prepare_quorum = PrePareQuroum::new(replica_number);
        let prepare_quorum = quorum::Quorum::PrePareQuroum(prepare_quorum);
        let commit_quorum = CommitQuroum::new(replica_number);
        let commit_quorum = quorum::Quorum::CommitQuroum(commit_quorum);
        let blockchain = blockchain::Blockchain::new(id.clone());
        let agreeing_block = block::Block::default();
        let mut in_memory_db = InMemoryDB::new(EmptyDBTyped::new());
        let spec_id = revm::primitives::SpecId::TANGERINE;
        let warm_preloaded_addresses = HashSet::new();
        let mut journaled_state = JournaledState::new(spec_id, warm_preloaded_addresses);
        let file_path = "address.txt";
        let mut file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let line_count = reader.lines().count();
        for i in 0..line_count {
            let mut file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let address_str = reader.lines().nth(i).unwrap().unwrap();
            let address_str = address_str.strip_prefix("0x").unwrap();
            let address = Address::from_str(address_str).unwrap();
            let balance:U256 = "10000".parse().unwrap();
            let code_hash = FixedBytes::ZERO;
            let code = Bytecode::new();
            let account_info = AccountInfo::new(balance, 0, code_hash, code);
            in_memory_db.insert_account_info(address, account_info);
            journaled_state.load_account(address, &mut in_memory_db);
        }
        
        PBFT{id, socket, verifyingkeys, signing_key, verifying_key, view_channel_tx, current_prepare_quorum_certificate, prepare_quorum, commit_quorum, blockchain, agreeing_block, journaled_state, in_memory_db}
    }

    pub fn exchange_verifying_key(&self){
        let verifyingkey = self.verifying_key;
        let serialized_verifyingkey = verifyingkey.to_sec1_bytes();
        // let asd = serialized_verifyingkey.to_vec();
        // let qwe: VerifyingKey<Secp256k1> = VerifyingKey::from_sec1_bytes(&asd).unwrap();
        let socket = self.socket.clone();
        let mut socket = socket.lock().map_err(|poisoned| {
            error!("Socket is poisoned {:?}", poisoned);
        }).unwrap();
        let verifyingkey = message::Verifyingkey {
            id: self.id.to_string(),
            verifyingkey: serialized_verifyingkey.to_vec()
        };
        socket.broadcast(message::Message::Verifyingkey(verifyingkey))
    }

    pub fn store_verifyingkey(&mut self, id:types::Identity,verifyingkey: Vec<u8>) {
         self.verifyingkeys.insert(id,verifyingkey);

    }

    pub fn make_block(&mut self, mempool:Arc<Mutex<MemPool>>, view:types::View) {
        info!("make block start");
        // self.excute_transaction();
        let mut mempool = mempool.lock().map_err(|poisoned| {
            error!("mempool is poisoned {:?}", poisoned);
        }).unwrap();
        let config = load_config().unwrap();
        let batch_size = config.batch_size;
        let payload = mempool.payload(batch_size);
        info!("payload size {:?}", payload.len());
        if view == 1 {
            let parent_block_id = "0000000000000000000000000000000000000000000000000000000000000000";
            let block_height = 1;
            self.process_block(payload, view, block_height, parent_block_id.to_string())
        } else {
            let parent_block_id = self.agreeing_block.block_id.to_string();
            let block_height = self.current_prepare_quorum_certificate + 1;
            self.process_block(payload, view, block_height, parent_block_id)
        }
        
    }

    pub fn excute_transaction(&mut self, payload:Vec<Transaction>) -> u64 {
        for transaction in payload {
            let from = Address::from_str(&transaction.from).unwrap();
            let to = Address::from_str(&transaction.to).unwrap();
            let _ = self.journaled_state.transfer(&from, &to, U256::from(5000), &mut self.in_memory_db).unwrap();
        }
        let mut hasher = DefaultHasher::new();
        let _ = &self.journaled_state.journal.hash(&mut hasher);
        hasher.finish()
    }

    pub fn process_block(&mut self, payload:Vec<Transaction>, view:types::View, block_height:types::BlockHeight, parent_block_id:types::BlockID) {
        let state = self.excute_transaction(payload.clone());
        let block_without_signature: BlockWithoutSignature = block::BlockWithoutSignature {
            payload,
            view,
            block_height,
            proposer: 1.to_string(),
            parent_block_id: parent_block_id.clone(),
            state
        };
        let block_id = make_block_id(&block_without_signature);
        let serialized_block = serde_json::to_vec(&block_without_signature).map_err(|e| {
            error!("Serialized block without signature {:?}", e)
        }).unwrap();
        let signature = make_signature(self.signing_key.clone(), &serialized_block);
        let signature = Signature::to_vec(&signature);
        let payload = block_without_signature.payload;
        let id = &self.id;
        let block = block::Block{
            block_id,
            payload,
            signature,
            block_height,
            view,
            proposer: id.to_string(),
            parent_block_id,
            state

        };
        self.agreeing_block = block.clone();
        let preprepare_message = message::PrePrePare {
            block
        };
        let socket = self.socket.clone();
        match socket.try_lock() {
            Ok(mut socket) => {
                socket.broadcast(message::Message::PrePrePare(preprepare_message))
            },
            Err(e) => {
                error!("Mutex is {:?}", e);
            },
        };
        // let mut socket = socket.lock().map_err(|poisoned| {
        //     error!("Socket is poisoned {:?}", poisoned);
        // }).unwrap();
       
    }


    pub fn process_preprepare(&mut self,  preprepare_message: PrePrePare) {
        info!("start processing block");
        let proposer_publickey = self.verifyingkeys.get(&preprepare_message.block.proposer).unwrap();
        let message = message::Message::PrePrePare(preprepare_message.clone());
        if verify_signature(proposer_publickey.to_owned(), message) {
            info!("success to verify block");
        } else {
            info!("fail to verify block");
            return
        }

        let state = self.excute_transaction(preprepare_message.block.payload);
        if !state.eq(&preprepare_message.block.state) {
            error!("state dosen't match");
            return
        }

        let view = preprepare_message.block.view.clone();
        let block_height = preprepare_message.block.block_height.clone();
        let prepare_message_without_signature = message::PrePareWithoutSignature {
            view,
            block_height,
            proposer: self.id.clone()
        };
        
        let serialized_message = serde_json::to_vec(&prepare_message_without_signature).map_err(|e| {
            error!("Serialized prepare message without signature {:?}", e)
        }).unwrap();
        let signature = make_signature(self.signing_key.clone(), &serialized_message);
        let signature = Signature::to_vec(&signature);
        let prepare_message = message::PrePare {
            view,
            block_height,
            proposer: self.id.clone(),
            signature
        };
        let socket = self.socket.clone();
        match socket.try_lock() {
            Ok(mut socket) => {
                socket.broadcast(message::Message::PrePare(prepare_message.clone()))
            },
            Err(e) => {
                error!("Mutex is {:?}", e);
            },
        };
        Self::process_prepare(self, prepare_message);
        // sleep(time::Duration::new(1, 0));
        // let rt = Runtime::new().unwrap();
        // rt.block_on(self.advance_view());
        

    }

    pub fn process_prepare(&mut self, prepare_message: PrePare) {
        info!("{:?} start process_prepare", self.id.clone());
        if self.id != prepare_message.proposer {
            let proposer_publickey = self.verifyingkeys.get(&prepare_message.proposer).expect("fail to get proposer's publickey");
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
            let proposer_publickey = self.verifyingkeys.get(&commit_message.proposer).expect("fail to get proposer's publickey");
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
                let commit_message_without_signature = message::CommitWithoutSignature {
                    view,
                    block_height,
                    proposer: self.id.clone()
                };
                
                let serialized_message = serde_json::to_vec(&commit_message_without_signature).map_err(|e| {
                    error!("Serialized commit message without signature {:?}", e)
                }).unwrap();
                
                let signature = make_signature(self.signing_key.clone(), &serialized_message);
                let signature = Signature::to_vec(&signature);
                
                let commit_message = message::Commit {
                    view,
                    block_height,
                    proposer: self.id.clone(),
                    signature
                };
                let socket = self.socket.clone();
                match socket.try_lock() {
                    Ok(mut socket) => {
                        socket.broadcast(message::Message::Commit(commit_message.clone()))
                    },
                    Err(e) => {
                        error!("Mutex is {:?}", e);
                    },
                };
                self.process_commit(commit_message);
            },
            Message::Commit(commit_message) => {
                let block = &self.agreeing_block;
                self.blockchain.commit_block(block.clone(), self.id.clone());
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
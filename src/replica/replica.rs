
use ecdsa::{SigningKey, VerifyingKey};
use k256::Secp256k1;
use rand_core::OsRng;
use revm::db::{self, Database, EmptyDB};
use revm::inspectors::NoOpInspector;
use revm::primitives::Address;
use revm::{inspector_handle_register, Evm, InMemoryDB};
use ring::rand::*;
use tokio::sync::mpsc::{
    Receiver, Sender, channel
};
use tokio::runtime::Runtime;
use std::sync::{Arc, Mutex};
use std::thread::{self, sleep};
use std::time::Duration;
use crate::blockchain::block;
use crate::consensus::{self, Consensus};
use crate::crypto::{self, Crypto};
use crate::mempool::MemPool;
use crate::{load_config, mempool, message, transport::*};
use crate::message::{PrePare, PrePrePare, Verifyingkey, Commit};
use crate::types::*;
use crate::http::*;
use log::{info, error};


pub struct Replica{
    id: Identity,
    transport: transport::Transport,
    consensus: consensus::Consensus,
    http: http::HTTP,
    transaction_channel_tx: Sender<message::Transaction>,
    transaction_channel_rx: Receiver<message::Transaction>,
    view_channel_rx: Receiver<types::View>,
    start_signal_tx: Sender<bool>,
    start_signal_rx: Receiver<bool>,
    mempool: mempool::MemPool,
}
impl Replica {

    pub fn new(id: i32, consensus:String, replica_number:i32) -> Replica {
        info!(" [{}] {} Replica started", id, consensus);
        let (transaction_channel_tx, transaction_channel_rx) = channel::<message::Transaction>(1000);
        let (view_channel_tx, view_channel_rx) = channel::<types::View>(1000);
        let (start_signal_tx, start_signal_rx) = channel::<bool>(1);
        let transport = Transport::new(id);
        let crypto = Crypto::new();
        let mempool = MemPool::new();
        let consensus = match consensus.as_str() {
            "pbft" => Some(consensus::Consensus::PBFT(
                consensus::PBFT::new(id, crypto.signing_key, crypto.verifying_key, view_channel_tx, replica_number)
            )),
            _ => {
                error!("Consensus name is not matched");
                None
            }
        };
        let port = 10000 + id;
        let id = id.to_string();
        let consensus = consensus.expect("Consensus name is not matched");
        let http = HTTP {
            host: String::from("127.0.0.1"),
            port: port.to_string(),
            workers: 4
        };
        Replica {id, transport, consensus, http, transaction_channel_tx, transaction_channel_rx, view_channel_rx, start_signal_tx, start_signal_rx, mempool}
    }

    pub fn start(self) {
        info!(" [{}] strat listening TCP port {:?}", self.id, self.transport.connection().local_addr().unwrap());
        let mut handles = vec![];
        let handle = thread::spawn(move|| {
            let _ = http::start_server(self.http, self.transaction_channel_tx);
        });
        handles.push(handle);
        let consensus = Arc::new(Mutex::new(self.consensus));
        let consensus_for_transaction = Arc::clone(&consensus);
        let consensus_for_advance_view = Arc::clone(&consensus);
        let mempool = Arc::new(Mutex::new(self.mempool));
        let mempool_for_generate_payload = Arc::clone(&mempool);
        let mempool_for_advance_view = Arc::clone(&mempool);
        let rt = Runtime::new().unwrap();
        rt.spawn(async move{
            Self::handle_transaction(self.start_signal_tx, mempool_for_generate_payload.clone(), self.transaction_channel_rx).await;
        });
        
        let id = Arc::new(self.id.clone());
        let handle = thread::spawn(move|| {
            for stream in self.transport.connection().incoming() {
                let id_clone = id.clone();
                let consensus = Arc::clone(&consensus_for_transaction);
                match stream {
                    Ok(stream) => {
                        thread::spawn(move|| {
                            Transport::handle_connection(consensus.clone(), id_clone, stream);
                        });
                    }
                    Err(err) => {
                        error!("connection refused {}", err);
                    }
                }
            }
        });
        handles.push(handle);
        Self::exchange_verifying_key(consensus);
        let view:types::View = 1;
        rt.spawn(async move{
            Self::handle_advance_view(self.id, self.start_signal_rx,  consensus_for_advance_view, mempool_for_advance_view, self.view_channel_rx, view).await;
        });
        for handle in handles{
            handle.join().unwrap()
        }
    }

    
    pub fn exchange_verifying_key(consensus:Arc<Mutex<Consensus>>) {
        let consensus = consensus.lock().unwrap();
        consensus.exchange_verifying_key();
    }

    pub async fn handle_advance_view(id:types::Identity, mut start_signal_handler: Receiver<bool>, consensus:Arc<Mutex<Consensus>>, mempool:Arc<Mutex<mempool::MemPool>>, mut view_hanlder: Receiver<types::View>, view:types::View) {
        if view == 1 {
            if id == 1.to_string() {
                let asd = start_signal_handler.recv().await;
                let mut consensus = consensus.lock().unwrap();
                consensus.make_block(mempool.clone(), view);
            }
        }
        while let Some(view) = view_hanlder.recv().await {
            let mut consensus = consensus.lock().unwrap();
            consensus.make_block(mempool.clone(), view)
        }
        
    }
    
    pub async fn handle_transaction(signal_tx: Sender<bool>,mempool:Arc<Mutex<mempool::MemPool>>, mut tx_handler: Receiver<message::Transaction>) {
        info!("handle_transaction started");
        let mut count = 0;
        while let Some(transaction) = tx_handler.recv().await {
            if count == 0 {
                let _ = signal_tx.send(true).await;
                count += 1
            }
            let mut mempool = mempool.lock().unwrap();
            mempool.add_transaction(transaction);
        }
    }

    pub fn handle_verifyingkey_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: message::Verifyingkey) {
        info!("[{:?}] publickey Message {:?}", id,message.id);
        let mut consensus = consensus.lock().unwrap();
        consensus.store_verifyingkey(message.id,message.verifyingkey)
        
    }
    pub fn handle_preprepare_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: PrePrePare) {
        info!("[{:?}] handle PrePrePare Message {:?}", id,message.block.view);
        let mut consensus = consensus.lock().unwrap();
        consensus.process_preprepare(message)
        
    }

    pub fn handle_prepare_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: PrePare) {
        info!("[{:?}] handle PrePare Message {:?}", id,message.view);
        let mut consensus = consensus.lock().unwrap();
        consensus.process_prepare(message)
    }

    pub fn handle_commit_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: Commit) {
        info!("[{:?}] handle Commit Message {:?}", id,message.view);
        let mut consensus = consensus.lock().unwrap();
        consensus.process_commit(message)
    }
}

fn generate_random_address() -> Address {
    let sigingkey = SigningKey::random(&mut OsRng);
    let verifyingkey = VerifyingKey::from(&sigingkey);
    Address::from_public_key(&verifyingkey)
}

use ring::signature::EcdsaKeyPair;
use tokio::sync::mpsc::{
    Receiver, Sender, channel
};
use tokio::runtime::Runtime;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::blockchain::block;
use crate::consensus::{self, Consensus};
use crate::crypto::{self, Crypto};
use crate::mempool::MemPool;
use crate::{load_config, mempool, message, transport::*};
use crate::message::{PrePare, PrePrePare, PublicKey, Commit};
use crate::types::*;
use crate::http::*;
use log::{info, error};


pub struct Replica{
    id: Identity,
    transport: transport::Transport,
    consensus: consensus::Consensus,
    http: http::HTTP,
    tx: Sender<message::Transaction>,
    rx: Receiver<message::Transaction>,
    mempool: mempool::MemPool,
}
impl Replica {

    pub fn new(id: i32, consensus:String) -> Replica {
        info!(" [{}] {} Replica started", id, consensus);
        let transport = Transport::new(id);
        let crypto = Crypto::new();
        let mempool = MemPool::new();
        let consensus = match consensus.as_str() {
            "pbft" => Some(consensus::Consensus::PBFT(
                consensus::PBFT::new(id, crypto.key_pair)
            )),
            _ => {
                error!("Consensus name is not matched");
                None
            }
        };
        let port = 10000 + id;
        let id = id.to_string();
        let consensus = consensus.expect("Consensus name is not matched");
        let (tx, rx) = channel::<message::Transaction>(100);
        let http = HTTP {
            host: String::from("127.0.0.1"),
            port: port.to_string(),
            workers: 4
        };
        Replica {id, transport, consensus, http, tx, rx, mempool}
    }

    pub fn start(self) {
        info!(" [{}] strat listening TCP port {:?}", self.id, self.transport.connection().local_addr().unwrap());
        
        thread::spawn(move|| {
            let _ = http::start_server(self.http, self.tx);
        });
        let consensus = Arc::new(Mutex::new(self.consensus));
        let consensus_for_transaction = Arc::clone(&consensus);
        let consensus_for_advance_view = Arc::clone(&consensus);
        let mempool = Arc::new(Mutex::new(self.mempool));
        let mempool_for_generate_payload = Arc::clone(&mempool);
        let rt = Runtime::new().unwrap();
        rt.spawn(async move{
            Self::handle_transaction(mempool_for_generate_payload.clone(), self.rx).await;
        });
        Self::exchange_publickey(consensus);
        Self::advance_view(consensus_for_advance_view, mempool);
        
        let id = Arc::new(self.id);
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
    }

    
    pub fn exchange_publickey(consensus:Arc<Mutex<Consensus>>) {
        let consensus = consensus.lock().unwrap();
        consensus.exchange_publickey();
    }

    pub fn advance_view(consensus:Arc<Mutex<Consensus>>, mempool:Arc<Mutex<mempool::MemPool>>) {
        let consensus = consensus.lock().unwrap();
        consensus.make_block(mempool.clone());
    }
    
    pub async fn handle_transaction(mempool:Arc<Mutex<mempool::MemPool>>, mut tx_handler: Receiver<message::Transaction>) {
        info!("handle_transaction started");

        while let Some(transaction) = tx_handler.recv().await {
            let mut mempool = mempool.lock().unwrap();
            mempool.add_transaction(transaction);
        }
    }

    pub fn handle_publickey_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: PublicKey) {
        info!(" [{:?}] publickey Message {:?}", id,message);
        let mut consensus = consensus.lock().unwrap();
        consensus.store_publickey(message.id,message.publickey)
        
    }
    pub fn handle_preprepare_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: PrePrePare) {
        info!(" [{:?}] PrePrePare Message {:?}", id,message);
        let consensus = consensus.lock().unwrap();
        consensus.process_preprepare(message)
        
    }

    pub fn handle_prepare_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: PrePare) {
        info!(" [{:?}] PrePare Message {:?}", id,message);
        let consensus = consensus.lock().unwrap();
        consensus.process_prepare(message)
    }

    pub fn handle_commit_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: Commit) {
        info!(" [{:?}] COMMIT Message {:?}", id,message);
        let consensus = consensus.lock().unwrap();
        consensus.process_commit(message)
    }
}
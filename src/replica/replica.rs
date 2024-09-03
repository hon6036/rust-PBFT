
use tokio::sync::mpsc::{
    Receiver, Sender, channel
};
use tokio::runtime::Runtime;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::consensus::{self, Consensus};
use crate::crypto::{self, Crypto};
use crate::mempool::MemPool;
use crate::{load_config, mempool, message, transport::*};
use crate::message::{PrePrePare, PrePare, COMMIT};
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
    crypto: crypto::Crypto
}
impl Replica {

    pub fn new(id: i32, consensus:String) -> Replica {
        let transport = Transport::new(id);
        
        info!(" [{}] {} Replica started", id, consensus);
        let consensus = match consensus.as_str() {
            "pbft" => Some(consensus::Consensus::PBFT(
                consensus::PBFT::new(id)
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
        let mempool = MemPool::new();
        let crypto = Crypto::new();
        Replica {id, transport, consensus, http, tx, rx, mempool, crypto}
        
    }

    pub fn start(self) {
        info!(" [{}] strat listening TCP port {:?}", self.id, self.transport.connection().local_addr().unwrap());
        
        thread::spawn(move|| {
            let _ = http::start_server(self.http, self.tx);
        });
        let consensus = Arc::new(Mutex::new(self.consensus));
        let consensus_for_transaction = Arc::clone(&consensus);
        let mempool = Arc::new(Mutex::new(self.mempool));
        let mempool_for_generate_payload = Arc::clone(&mempool);
        let rt = Runtime::new().unwrap();
        rt.spawn(async move{
            Self::handle_transaction(mempool_for_generate_payload.clone(), consensus_for_transaction.clone(),self.rx).await;
        });
        let block = Self::make_block(mempool.clone());
        let id = Arc::new(self.id);
        for stream in self.transport.connection().incoming() {
            let id_clone = id.clone();
            let consensus = Arc::clone(&consensus);
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
    
    pub async fn handle_transaction(mempool:Arc<Mutex<mempool::MemPool>>,consensus:Arc<Mutex<Consensus>>, mut tx_handler: Receiver<message::Transaction>) {
        info!("handle_transaction started");

        while let Some(transaction) = tx_handler.recv().await {
            let mut mempool = mempool.lock().unwrap();
            mempool.add_transaction(transaction);
            let consensus = consensus.lock().unwrap();
            
        }
    }

    pub fn make_block(mempool:Arc<Mutex<mempool::MemPool>>) {
        let mut mempool = mempool.lock().unwrap();
        let config = load_config().unwrap();
        let batch_size = config.batch_size;
        let payload = mempool.payload(batch_size);
    }

    pub fn handle_preprepare_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: PrePrePare) {
        info!(" [{:?}] PrePrePare Message {:?}", id,message);
        let consensus = consensus.lock().unwrap();
        consensus.process_preprepare()
        
    }

    pub fn handle_prepare_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: PrePare) {
        info!(" [{:?}] PrePare Message {:?}", id,message);
        let consensus = consensus.lock().unwrap();
        // consensus.process_prepare()
    }

    pub fn handle_commit_message(consensus:Arc<Mutex<Consensus>>, id: Arc<String>, message: COMMIT) {
        info!(" [{:?}] COMMIT Message {:?}", id,message);
        let consensus = consensus.lock().unwrap();
        consensus.process_commit()
    }
}
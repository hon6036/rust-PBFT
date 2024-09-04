
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
            Self::handle_transaction(mempool_for_generate_payload.clone(), self.rx).await;
        });
        Self::advance_view(consensus_for_transaction, mempool, self.crypto.key_pair);
        
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

    pub fn advance_view(consensus:Arc<Mutex<Consensus>>, mempool:Arc<Mutex<mempool::MemPool>>, keypair:EcdsaKeyPair ) {
        let consensus = consensus.lock().unwrap();
        consensus.make_block(mempool.clone(), keypair);
    }
    
    pub async fn handle_transaction(mempool:Arc<Mutex<mempool::MemPool>>, mut tx_handler: Receiver<message::Transaction>) {
        info!("handle_transaction started");

        while let Some(transaction) = tx_handler.recv().await {
            let mut mempool = mempool.lock().unwrap();
            mempool.add_transaction(transaction);
        }
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
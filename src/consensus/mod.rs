pub mod pbft;
use log::info;
pub use pbft::*;
use ring::signature::EcdsaKeyPair;
use crate::{crypto, mempool::*, message::*, types::types};
use std::sync::{Arc, Mutex};
pub enum Consensus {
    PBFT(PBFT)
}

impl Consensus {
    pub fn exchange_publickey(&self) {
        match self {
            Consensus::PBFT(pbft) => pbft.exchange_publickey()
        }
    }
    pub fn make_block(&self, mempool:Arc<Mutex<MemPool>>, view:types::View) {
        match self {
            Consensus::PBFT(pbft) => pbft.make_block(mempool, view)
        }
    }
    pub fn store_publickey(&mut self, id:types::Identity, publickey: Vec<u8>) {
        match self {
            Consensus::PBFT(pbft) => pbft.store_publickey(id,publickey)
        }
    }
    pub fn process_preprepare(&mut self, message: PrePrePare) {
        match self {
            Consensus::PBFT(pbft) => pbft.process_preprepare(message)
        }
    }
    pub fn process_prepare(&mut self, message: PrePare) {
        match self {
            Consensus::PBFT(pbft) => pbft.process_prepare(message)
        }
    }
    pub fn process_commit(&mut self, message: Commit) {
        match self {
            Consensus::PBFT(pbft) => pbft.process_commit(message)
        }
    }
}
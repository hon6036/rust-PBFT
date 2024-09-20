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
    pub fn exchange_verifying_key(&self) {
        match self {
            Consensus::PBFT(pbft) => pbft.exchange_verifying_key()
        }
    }
    pub fn make_block(&mut self, mempool:Arc<Mutex<MemPool>>, view:types::View) {
        match self {
            Consensus::PBFT(pbft) => pbft.make_block(mempool, view)
        }
    }
    pub fn store_verifyingkey(&mut self, id:types::Identity, verifyingkey: Vec<u8>) {
        match self {
            Consensus::PBFT(pbft) => pbft.store_verifyingkey(id,verifyingkey)
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
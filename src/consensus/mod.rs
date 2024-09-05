pub mod pbft;
use log::info;
pub use pbft::*;
use ring::signature::EcdsaKeyPair;
use crate::{crypto, mempool::*, types::types};
use std::sync::{Arc, Mutex};
pub enum Consensus {
    PBFT(PBFT)
}

impl Consensus {
    pub fn exchange_publickey(&self, key_pair:&EcdsaKeyPair) {
        match self {
            Consensus::PBFT(pbft) => pbft.exchange_publickey(&key_pair)
        }
    }
    pub fn make_block(&self, mempool:Arc<Mutex<MemPool>>, key_pair:EcdsaKeyPair) {
        match self {
            Consensus::PBFT(pbft) => pbft.make_block(mempool, key_pair)
        }
    }
    pub fn store_publickey(&mut self, id:types::Identity, publickey: Vec<u8>) {
        match self {
            Consensus::PBFT(pbft) => pbft.store_publickey(id,publickey)
        }
    }
    pub fn process_preprepare(&self) {
        match self {
            Consensus::PBFT(pbft) => pbft.process_preprepare()
        }
    }
    pub fn process_prepare(&self) {
        match self {
            Consensus::PBFT(pbft) => pbft.process_prepare()
        }
    }
    pub fn process_commit(&self) {
        match self {
            Consensus::PBFT(pbft) => pbft.process_commit()
        }
    }
}
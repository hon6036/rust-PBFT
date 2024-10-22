use std::{sync::{Arc, Mutex}, time::{SystemTime, UNIX_EPOCH}};

use log::info;
use serde::Serialize;
use time::Duration;

use crate::message::{self, Transaction};

pub struct MemPool {
    transactions: Vec<message::Transaction>
}

impl MemPool {
    pub fn new() -> MemPool {
        let transactions: Vec<message::Transaction> = Vec::new(); 
        MemPool{transactions: transactions }
    }

    pub fn add_transaction(&mut self,mut transaction:message::Transaction) {
        transaction.timestamp = SystemTime::now();
        self.transactions.push(transaction);
    }

    pub fn payload(&mut self, batch_size:usize) -> Vec<message::message::Transaction> {
        let mempool_size = self.transactions.len();
        if mempool_size < batch_size {
            let payload = self.transactions.drain(0..mempool_size).collect();
            payload
        } else {
            let payload = self.transactions.drain(0..batch_size).collect();
            payload
        }
    }
}
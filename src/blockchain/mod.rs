pub mod block;
use std::{collections::HashMap, fs::File, time::{Duration, SystemTime}};

pub use block::*;
use csv::Writer;
use log::info;
use serde::Deserialize;

use crate::{benchmark::Benchmark, message, types::types};
pub struct Blockchain {
    database:HashMap<types::BlockID,block::Block>,
    benchmark: Benchmark,
    total_committed_transaction: u64,
    total_latency: f32,
    start_time: SystemTime,
    csv_file: Writer<File>
}

impl Blockchain {
    pub fn new(id:String) -> Blockchain {
        let database = HashMap::new();
        let benchmark = Benchmark::new();
        let wtr = Writer::from_path("./logs/result.csv").unwrap();
        Blockchain{
            database,
            benchmark,
            total_committed_transaction:0,
            total_latency:0.0,
            start_time: SystemTime::now(),
            csv_file: wtr
        }
    }

    pub fn commit_block(&mut self, block:Block, id:String) {
        let block_id = block.block_id.clone();
        if id.eq("1") {
            for transaction in block.payload.clone().iter() {
                self.total_committed_transaction += 1;
                self.total_latency += SystemTime::elapsed(&transaction.timestamp).unwrap().as_secs() as f32;
            }
            info!("total_committed_transaction {:?}", self.total_committed_transaction);
            if self.total_committed_transaction != 0 {
                self.benchmark.tps = self.total_committed_transaction/ SystemTime::elapsed(&self.start_time).unwrap().as_secs();
                self.benchmark.latency = self.total_latency / self.total_committed_transaction as f32;
            }
            self.csv_file.serialize(&self.benchmark);
            self.csv_file.flush();
        }
        self.database.insert(block_id, block);
    }

}
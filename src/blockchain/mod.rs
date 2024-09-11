pub mod block;
use std::collections::HashMap;

pub use block::*;

use crate::{message, types::types};
pub struct Blockchain {
    database:HashMap<types::BlockID,block::Block>
}

impl Blockchain {
    pub fn new(id:String) -> Blockchain {
        let database = HashMap::new();

        Blockchain{
            database
        }
    }

    pub fn commit_block(&mut self, block:Block) {
        let block_id = block.block_id.clone();
        self.database.insert(block_id, block);
    }

}
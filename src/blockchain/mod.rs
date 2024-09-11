pub mod block;
pub use block::*;
use rocksdb::DB;

use crate::types::types;
pub struct Blockchain {
    database:DBCommon
}

impl Blockchain {
    pub fn new() -> Blockchain {
        let path = "./blockchain";
        let db = DB::open_default(path).unwrap();

        Blockchain{
            database:db
        }
    }

    pub fn commit_block(self, block:Block) {
        self.database.put(block.block_id, block);
    }

}
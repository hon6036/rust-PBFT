use crate::{message, types};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    pub block_id: types::BlockID,
    pub block_height: types::BlockHeight,
    pub payload: Vec<message::Transaction>,
    pub view: types::View,
    pub signature: Vec<u8>,
    pub proposer: types::Identity
}
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockWithoutSignature {
    pub payload: Vec<message::Transaction>,
    pub view: types::View,
    pub block_height: types::BlockHeight,
    pub proposer: types::Identity
}
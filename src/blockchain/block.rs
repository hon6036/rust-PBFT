use crate::{message, types};
use ring::signature::Signature;
pub struct Block {
    block_id: types::BlockID,
    payload: Vec<message::Transaction>,
    view: types::View,
    signature: Signature,
    proposer: types::Identity
}

pub struct BlockWithoutSignature {
    block_id: types::BlockID,
    payload: Vec<message::Transaction>,
    view: types::View,
    proposer: types::Identity
}
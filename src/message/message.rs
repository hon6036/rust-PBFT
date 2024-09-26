use crate::{blockchain::block, types::*};
use ring::signature::UnparsedPublicKey;
use serde::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize)]
pub struct Verifyingkey {
    pub id: types::Identity,
    pub verifyingkey: Vec<u8>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrePrePare {
    pub block: block::Block
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrePare {
    pub view: types::View,
    pub block_height: types::BlockHeight,
    pub proposer: types::Identity,
    pub signature: Vec<u8>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PrePareWithoutSignature {
    pub view: types::View,
    pub block_height: types::BlockHeight,
    pub proposer: types::Identity,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Commit{
    pub view: types::View,
    pub block_height: types::BlockHeight,
    pub proposer: types::Identity,
    pub signature: Vec<u8>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitWithoutSignature{
    pub view: types::View,
    pub block_height: types::BlockHeight,
    pub proposer: types::Identity
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    pub from: types::Identity,
    pub to: types::Identity,
    pub balance: i32
}
use crate::{blockchain::block, types::*};
use ring::signature::UnparsedPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    PublicKey(PublicKey),
    PrePrePare(PrePrePare),
    PrePare(PrePare),
    Commit(Commit)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicKey {
    pub id: types::Identity,
    pub publickey: Vec<u8>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrePrePare {
    pub block: block::Block
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    sender: types::Identity,
    receiver: types::Identity,
    balance: i32
}
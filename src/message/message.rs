use crate::{blockchain::block, types::*};
use ring::signature::UnparsedPublicKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    PublicKey(PublicKey),
    PrePrePare(PrePrePare),
    PrePare(PrePare),
    COMMIT(COMMIT)
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
    pub id: types::Identity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct COMMIT{
    pub view: types::View,
    pub block_height: types::BlockHeight,
    pub id: types::Identity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    sender: types::Identity,
    receiver: types::Identity,
    balance: i32
}
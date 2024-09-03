use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    PrePrePare(PrePrePare),
    PrePare(PrePare),
    COMMIT(COMMIT)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrePrePare {
    pub view: types::View,
    pub block_height: types::BlockHeight,
    pub id: types::Identity,
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
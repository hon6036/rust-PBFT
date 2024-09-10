pub mod message;
pub use message::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    PublicKey(PublicKey),
    PrePrePare(PrePrePare),
    PrePare(PrePare),
    Commit(Commit)
}
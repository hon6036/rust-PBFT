pub mod message;
pub use message::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    Verifyingkey(Verifyingkey),
    PrePrePare(PrePrePare),
    PrePare(PrePare),
    Commit(Commit)
}
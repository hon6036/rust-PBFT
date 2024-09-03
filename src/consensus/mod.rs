pub mod pbft;
pub use pbft::*;

pub enum Consensus {
    PBFT(PBFT)
}

impl Consensus {
    pub fn process_preprepare(&self) {
        match self {
            Consensus::PBFT(pbft) => pbft.process_preprepare()
        }
    }
    pub fn process_prepare(&self) {
        match self {
            Consensus::PBFT(pbft) => pbft.process_prepare()
        }
    }
    pub fn process_commit(&self) {
        match self {
            Consensus::PBFT(pbft) => pbft.process_commit()
        }
    }
}
use std::collections::HashMap;

use crate::{message::message, types::types};

pub struct PrePareQuroum {
    pub collection: HashMap<types::View, HashMap<types::Identity,message::PrePare>>,
    pub replica_number: usize
}



impl PrePareQuroum {
    pub fn new(replica_number:i32) -> Self {
        let replica_number: usize = usize::try_from(replica_number).unwrap();
        Self {
            collection: HashMap::new(),
            replica_number: replica_number
        }
    }
}
pub struct CommitQuroum {
    pub collection: HashMap<types::View, HashMap<types::Identity,message::Commit>>,
    pub replica_number: usize
}

impl CommitQuroum {
    pub fn new(replica_number:i32) -> Self {
        let replica_number: usize = usize::try_from(replica_number).unwrap();
        Self {
            collection: HashMap::new(),
            replica_number: replica_number
        }
    }
}
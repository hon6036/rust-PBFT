pub mod preparequorum;
use std::collections::HashMap;

use log::info;
pub use preparequorum::*;
use either::*;
use crate::message;

pub enum Quorum {
    PrePareQuroum(PrePareQuroum),
    CommitQuroum(CommitQuroum)
}

impl Quorum {
    pub fn add(message:message::Message, quorum:&mut Quorum) -> Either<message::Message,bool> {
        match message {
            message::Message::PrePare(message) => {
                if let Quorum::PrePareQuroum(ref mut quorum) = quorum {
                    let check_2f = Self::check_2f(&message, quorum);

                    if check_2f.is_right() {
                        info!("prepare message aleady satisfied");
                        return either::Right(false)
                    } else {
                        let message = check_2f.left().unwrap();
                        let entry = quorum.collection.entry(message.view.clone()).or_insert_with(HashMap::new);
                        entry.insert(message.proposer.clone(), message.clone());
                        let check_2f = Self::check_2f(&message, quorum);
                        if check_2f.is_left() {
                            info!("prepare message is not enuough to continue process");
                            return either::Right(false)
                        } else {
                            info!("prepare message is enough to continue process");
                            return either::Left(message::Message::PrePare(message))
                        }
                    }
                } else {
                    return either::Right(false)
                }
            },
            message::Message::Commit(message) => {
                if let Quorum::CommitQuroum(ref mut quorum) = quorum {
                    let check_2f = Self::check_2f_plus_1(&message, quorum);

                    if check_2f.is_right() {
                        info!("prepare message aleady satisfied");
                        return either::Right(false)
                    } else {
                        let message = check_2f.left().unwrap();
                        let entry = quorum.collection.entry(message.view.clone()).or_insert_with(HashMap::new);
                        entry.insert(message.proposer.clone(), message.clone());
                        let check_2f = Self::check_2f_plus_1(&message, quorum);
                        if check_2f.is_left() {
                            info!("prepare message is not enuough to continue process");
                            return either::Right(false)
                        } else {
                            info!("prepare message is enough to continue process");
                            return either::Left(message::Message::Commit(message))
                        }
                    }
                } else {
                    return either::Right(false)
                }
            },
            _ => todo!(),
        }
    }

    pub fn check_2f(message:&message::PrePare, quorum:&mut PrePareQuroum) -> Either<message::PrePare, bool>  {
        let hash_map = quorum.collection.entry(message.view).or_insert_with(HashMap::new);
        if hash_map.len() >= (quorum.replica_number)*2/3 {
            either::Right(true)
        } else {
            either::Left(message.clone())
        }
    }

    pub fn check_2f_plus_1(message:&message::Commit, quorum:&mut CommitQuroum) -> Either<message::Commit, bool> {
        let hash_map = quorum.collection.entry(message.view).or_insert_with(HashMap::new);
        if hash_map.len() > (quorum.replica_number)*2/3 {
            either::Right(true)
        } else {
            either::Left(message.clone())
        }
    }
}
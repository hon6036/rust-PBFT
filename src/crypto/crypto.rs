use core::str;
use std::sync::{Arc, Mutex};

use log::info;
use ring:: {
    digest::{Context, Digest, SHA256}, rand, signature::{
        self, EcdsaKeyPair, EcdsaVerificationAlgorithm, KeyPair, UnparsedPublicKey, VerificationAlgorithm, ECDSA_P256_SHA256_ASN1_SIGNING, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING
    }
};
use serde::Serialize;

use crate::{blockchain::{block, BlockWithoutSignature}, message, types};


pub struct Crypto {
    pub(crate) key_pair: EcdsaKeyPair
}

impl Crypto {
    pub fn new() -> Crypto {
        let rng = rand::SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8.as_ref(), &rng).unwrap();
       Crypto{key_pair}
    }
}

pub fn make_signature(key_pair:&EcdsaKeyPair, serialized_message:&Vec<u8>) -> Vec<u8> {
    let rng = rand::SystemRandom::new();
    let signature = key_pair.sign(&rng, serialized_message).unwrap();
    
    signature.as_ref().to_vec()
}

pub fn verify_signature(proposer_publicekey:Vec<u8>, message:message::Message) -> bool {
    let proposer_publicekey:&[u8] = proposer_publicekey.as_ref();
    let proposer_publicekey = UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, proposer_publicekey);
    match message {
        message::Message::PrePrePare(message) => {
            let block_without_signature = block::BlockWithoutSignature {
                payload: message.block.payload,
                view: message.block.view,
                block_height: message.block.block_height,
                proposer: message.block.proposer,
                parent_block_id: message.block.parent_block_id
            };
            let serialized_block = serde_json::to_vec(&block_without_signature).unwrap();
            match proposer_publicekey.verify(&serialized_block, message.block.signature.as_ref()) {
                Ok(_) => true,
                Err(_) => {
                    // info!("Failed to verify Block: {:?}", e);
                    false
                }
            }
        },
        message::Message::PrePare(message) => {
            let message_without_signature = message::PrePareWithoutSignature {
                view: message.view,
                block_height: message.block_height,
                proposer: message.proposer,
            };
            let serialized_prepare_message = serde_json::to_vec(&message_without_signature).unwrap();
            match proposer_publicekey.verify(&serialized_prepare_message, message.signature.as_ref()) {
                Ok(_) => true,
                Err(_) => {
                    info!("Failed to verify prepare");
                    false
                },
            }
        },
        message::Message::Commit(message) => {
            let message_without_signature = message::CommitWithoutSignature {
                view: message.view,
                block_height: message.block_height,
                proposer: message.proposer,
            };
            let serialized_commit_message = serde_json::to_vec(&message_without_signature).unwrap();
            match proposer_publicekey.verify(&serialized_commit_message, message.signature.as_ref()) {
                Ok(_) => true,
                Err(_) => false,
            }
        },
        message::Message::PublicKey(_) => false,
    }

}

pub fn make_block_id(block_without_signature:&block::BlockWithoutSignature) -> String{
    let serialized_block = serde_json::to_vec(block_without_signature).unwrap();
    let mut context = Context::new(&SHA256);
    context.update(&serialized_block);
    let digest = context.finish();
    let block_id: String = digest.as_ref()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();
    block_id
}

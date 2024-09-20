use core::str;
use std::sync::{Arc, Mutex};
use k256::{Secp256k1};
use rand_core::OsRng;
use ecdsa::{signature::{Signer}, Signature, SigningKey, VerifyingKey};
use log::{error, info};
use digest::{Digest, Output};
use sha2::{Sha256};
use ring:: {
    digest::{Context, SHA256}, rand, signature::{
        self, EcdsaKeyPair, EcdsaVerificationAlgorithm, KeyPair, UnparsedPublicKey, VerificationAlgorithm, ECDSA_P256_SHA256_ASN1_SIGNING, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING
    }
};
use serde::Serialize;

use crate::{blockchain::{block, BlockWithoutSignature}, message, types};


pub struct Crypto {
    pub(crate) signing_key: SigningKey<Secp256k1>,
    pub(crate) verifying_key: VerifyingKey<Secp256k1>,
}

impl Crypto {
    pub fn new() -> Crypto {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);

        // let rng = rand::SystemRandom::new();
        // let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
        // let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8.as_ref(), &rng).unwrap();
       Crypto{signing_key, verifying_key}
    }
}

pub fn make_signature(mut signing_key:SigningKey<Secp256k1>, serialized_message:&Vec<u8>) -> Signature<Secp256k1>{
    let (signature, recovery_id) = SigningKey::sign_prehash_recoverable(&signing_key, serialized_message).unwrap();
    signature
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
        message::Message::Verifyingkey(_) => false,
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

use core::str;

use log::info;
use ring:: {
    digest::{Context, Digest, SHA256}, rand, signature::{
        self, EcdsaKeyPair, EcdsaVerificationAlgorithm, KeyPair, UnparsedPublicKey, VerificationAlgorithm, ECDSA_P256_SHA256_ASN1_SIGNING, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING
    }
};
use serde::Serialize;

use crate::{blockchain::block, message, types};


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

pub fn make_block_signature(key_pair:EcdsaKeyPair, block_without_signature:&block::BlockWithoutSignature) -> Vec<u8> {
    let serialized_block = serde_json::to_vec(&block_without_signature).unwrap();
    
    let rng = rand::SystemRandom::new();
    let signature = key_pair.sign(&rng, &serialized_block).unwrap();
    
    signature.as_ref().to_vec()
}

pub fn verify_signature(key_pair:EcdsaKeyPair, message:message::Message, signature:Vec<u8>) {
    let a = UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, key_pair.public_key());
    let message = bincode::serialize(&message).unwrap();
    match a.verify(&message, signature.as_ref()) {
        Ok(_) => info!("서명 검증 성공"),
        Err(e) => info!("실패 {:?}", e)
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

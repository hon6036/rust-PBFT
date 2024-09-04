use log::info;
use ring:: {
    digest::SHA256, rand, signature::{
        self, EcdsaKeyPair, EcdsaVerificationAlgorithm, KeyPair, UnparsedPublicKey, VerificationAlgorithm, ECDSA_P256_SHA256_ASN1_SIGNING, ECDSA_P256_SHA256_FIXED, ECDSA_P256_SHA256_FIXED_SIGNING
    }
};

use crate::types;


pub struct Crypto {
    key_pair: EcdsaKeyPair
}

impl Crypto {
    pub fn new() -> Crypto {
        let rng = rand::SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8.as_ref(), &rng).unwrap();
        // let public_key = key_pair.public_key().to_owned();
        let message = "message";
        let signature = key_pair.sign(&rng, message.as_bytes()).unwrap();
        let a = UnparsedPublicKey::new(&ECDSA_P256_SHA256_FIXED, key_pair.public_key());
        match a.verify(message.as_bytes(), signature.as_ref()) {
            Ok(_) => info!("서명 검증 성공"),
            Err(e) => info!("실패 {:?}", e)
        }
        Crypto{key_pair}
    }
}

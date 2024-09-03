use ring:: {
    rand, digest::SHA256,signature::{
        self, EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_ASN1_SIGNING
    }
};

use crate::types;


pub struct Crypto {
    key_pair: EcdsaKeyPair
}

impl Crypto {
    pub fn new() -> Crypto {
        let rng = rand::SystemRandom::new();
        let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &pkcs8.as_ref(), &rng).unwrap();
        // let public_key = key_pair.public_key().to_owned();
        Crypto{key_pair}
    }
}

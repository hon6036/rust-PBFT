use ecdsa::{SigningKey, VerifyingKey};
use k256::Secp256k1;
use rand_core::OsRng;
use revm::primitives::Address;

pub struct Client {
    pub(crate) signing_key: SigningKey<Secp256k1>,
    pub(crate) verifying_key: VerifyingKey<Secp256k1>,
    pub(crate) address: Address
}

impl Client {
    pub fn new() -> Client {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);
        let address = Address::from_public_key(&verifying_key);
        // let rng = rand::SystemRandom::new();
        // let pkcs8 = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng).unwrap();
        // let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &pkcs8.as_ref(), &rng).unwrap();
        Client{signing_key, verifying_key, address}
    }
}
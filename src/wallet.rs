use rand::rngs::OsRng;
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message};
use sha2::{Digest, Sha256};

use crate::crypto;

pub struct Wallet {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub address: String,
}

impl Wallet {
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let address = derive_address(&public_key);

        Wallet {
            secret_key,
            public_key,
            address,
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.serialize())
    }

    pub fn sign(&self, message: &[u8]) -> String {
        let secp = Secp256k1::new();
        let digest = sha256_digest(message);
        let msg = Message::from_digest(digest);
        let sig = secp.sign_ecdsa(&msg, &self.secret_key);
        hex::encode(sig.serialize_compact())
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&result);
    digest
}

fn derive_address(public_key: &PublicKey) -> String {
    let hash = crypto::hash_sha256(&public_key.serialize());
    hash[..40].to_string()
}

pub fn address_from_public_key_hex(pk_hex: &str) -> Result<String, String> {
    let bytes = hex::decode(pk_hex).map_err(|e| format!("Invalid hex: {}", e))?;
    let pk = PublicKey::from_slice(&bytes).map_err(|e| format!("Invalid public key: {}", e))?;
    Ok(derive_address(&pk))
}

pub fn verify_signature(public_key_hex: &str, message: &[u8], signature_hex: &str) -> bool {
    let secp = Secp256k1::new();

    let pk_bytes = match hex::decode(public_key_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let public_key = match PublicKey::from_slice(&pk_bytes) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    let sig_bytes = match hex::decode(signature_hex) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let signature = match secp256k1::ecdsa::Signature::from_compact(&sig_bytes) {
        Ok(s) => s,
        Err(_) => return false,
    };

    let digest = sha256_digest(message);
    let msg = Message::from_digest(digest);

    secp.verify_ecdsa(&msg, &signature, &public_key).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert_eq!(wallet.address().len(), 40);
        assert!(!wallet.public_key_hex().is_empty());
    }

    #[test]
    fn test_sign_and_verify() {
        let wallet = Wallet::new();
        let message = b"hello blockchain";

        let signature = wallet.sign(message);
        assert!(verify_signature(&wallet.public_key_hex(), message, &signature));

        // Wrong message should fail
        assert!(!verify_signature(&wallet.public_key_hex(), b"wrong message", &signature));
    }

    #[test]
    fn test_different_wallets_different_addresses() {
        let w1 = Wallet::new();
        let w2 = Wallet::new();
        assert_ne!(w1.address(), w2.address());
    }

    #[test]
    fn test_address_from_public_key_hex() {
        let wallet = Wallet::new();
        let addr = address_from_public_key_hex(&wallet.public_key_hex()).unwrap();
        assert_eq!(addr, wallet.address());
    }
}

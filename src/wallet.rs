use rand::rngs::OsRng;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::crypto;

pub struct Wallet {
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
    pub address: String,
}

/// Serializable wallet data for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub name: String,
    pub address: String,
    pub public_key: String,
    pub secret_key: String,
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

    pub fn from_secret_key_hex(sk_hex: &str) -> Result<Self, String> {
        let bytes = hex::decode(sk_hex).map_err(|e| format!("Invalid hex: {}", e))?;
        let secret_key =
            SecretKey::from_slice(&bytes).map_err(|e| format!("Invalid secret key: {}", e))?;
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        let address = derive_address(&public_key);

        Ok(Wallet {
            secret_key,
            public_key,
            address,
        })
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.public_key.serialize())
    }

    pub fn secret_key_hex(&self) -> String {
        hex::encode(self.secret_key.secret_bytes())
    }

    pub fn to_data(&self, name: &str) -> WalletData {
        WalletData {
            name: name.to_string(),
            address: self.address.clone(),
            public_key: self.public_key_hex(),
            secret_key: self.secret_key_hex(),
        }
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

/// Manages wallet persistence to a JSON file.
pub struct WalletStore;

impl WalletStore {
    pub fn data_dir() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".rusty-chain")
    }

    pub fn wallets_path() -> std::path::PathBuf {
        Self::data_dir().join("wallets.json")
    }

    pub fn blockchain_path() -> std::path::PathBuf {
        Self::data_dir().join("blockchain.json")
    }

    pub fn load_wallets() -> Vec<WalletData> {
        let path = Self::wallets_path();
        if !path.exists() {
            return Vec::new();
        }
        let data = std::fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    }

    pub fn save_wallets(wallets: &[WalletData]) -> Result<(), String> {
        let dir = Self::data_dir();
        std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create data dir: {}", e))?;
        let json = serde_json::to_string_pretty(wallets)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
        std::fs::write(Self::wallets_path(), json)
            .map_err(|e| format!("Failed to write: {}", e))?;
        Ok(())
    }

    pub fn find_wallet(name: &str) -> Option<Wallet> {
        let wallets = Self::load_wallets();
        wallets
            .iter()
            .find(|w| w.name == name)
            .and_then(|w| Wallet::from_secret_key_hex(&w.secret_key).ok())
    }

    pub fn find_wallet_by_address(address: &str) -> Option<(String, Wallet)> {
        let wallets = Self::load_wallets();
        wallets.iter().find(|w| w.address == address).and_then(|w| {
            Wallet::from_secret_key_hex(&w.secret_key)
                .ok()
                .map(|wallet| (w.name.clone(), wallet))
        })
    }
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
        assert!(verify_signature(
            &wallet.public_key_hex(),
            message,
            &signature
        ));

        // Wrong message should fail
        assert!(!verify_signature(
            &wallet.public_key_hex(),
            b"wrong message",
            &signature
        ));
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

    #[test]
    fn test_from_secret_key_hex() {
        let wallet = Wallet::new();
        let sk_hex = wallet.secret_key_hex();
        let restored = Wallet::from_secret_key_hex(&sk_hex).unwrap();
        assert_eq!(restored.address(), wallet.address());
        assert_eq!(restored.public_key_hex(), wallet.public_key_hex());
    }

    #[test]
    fn test_wallet_data_roundtrip() {
        let wallet = Wallet::new();
        let data = wallet.to_data("test");
        assert_eq!(data.name, "test");
        assert_eq!(data.address, wallet.address());

        let restored = Wallet::from_secret_key_hex(&data.secret_key).unwrap();
        assert_eq!(restored.address(), wallet.address());
    }

    #[test]
    fn test_verify_signature_wrong_public_key() {
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let message = b"hello";

        let signature = wallet1.sign(message);
        // wallet2's public key should not verify wallet1's signature
        assert!(!verify_signature(
            &wallet2.public_key_hex(),
            message,
            &signature
        ));
    }

    #[test]
    fn test_from_secret_key_hex_invalid_hex() {
        assert!(Wallet::from_secret_key_hex("not-valid-hex").is_err());
    }

    #[test]
    fn test_from_secret_key_hex_wrong_length() {
        assert!(Wallet::from_secret_key_hex("deadbeef").is_err());
    }

    #[test]
    fn test_address_from_public_key_hex_invalid() {
        assert!(address_from_public_key_hex("not-valid-hex").is_err());
    }

    #[test]
    fn test_verify_signature_invalid_hex_returns_false() {
        let message = b"hello";
        assert!(!verify_signature(
            "invalid-pk-hex",
            message,
            "invalid-sig-hex"
        ));
    }

    #[test]
    fn test_public_key_hex_is_66_bytes_compressed() {
        let wallet = Wallet::new();
        // Compressed secp256k1 public key is 33 bytes = 66 hex chars
        assert_eq!(wallet.public_key_hex().len(), 66);
    }

    #[test]
    fn test_secret_key_hex_is_64_chars() {
        let wallet = Wallet::new();
        assert_eq!(wallet.secret_key_hex().len(), 64);
    }
}

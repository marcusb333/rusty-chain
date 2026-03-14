use crate::blockchain::Blockchain;
use std::fs;

pub struct Store;

impl Store {
    /// Save blockchain to disk (JSON format)
    pub fn save_blockchain(blockchain: &Blockchain, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&blockchain.chain)
            .map_err(|e| format!("Serialization error: {}", e))?;

        fs::write(path, json).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Load blockchain from disk
    pub fn load_blockchain(path: &str) -> Result<Blockchain, String> {
        let json = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let chain =
            serde_json::from_str(&json).map_err(|e| format!("Deserialization error: {}", e))?;

        Ok(Blockchain {
            chain,
            difficulty: 2,
            transaction_pool: Default::default(),
        })
    }

    /// Print blockchain info to console
    pub fn print_blockchain(blockchain: &Blockchain) {
        println!("\n🔗 Blockchain State:");
        println!("   Total blocks: {}", blockchain.chain.len());

        for block in &blockchain.chain {
            println!(
                "\n   Block #{} | Txs: {} | Hash: {}...",
                block.header.index,
                block.transactions.len(),
                &block.hash[..16]
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_load_blockchain() {
        let blockchain = Blockchain::new();
        let test_path = "/tmp/test_blockchain.json";

        // Save
        Store::save_blockchain(&blockchain, test_path).unwrap();
        assert!(std::path::Path::new(test_path).exists());

        // Load
        let loaded = Store::load_blockchain(test_path).unwrap();
        assert_eq!(loaded.chain.len(), blockchain.chain.len());

        // Cleanup
        fs::remove_file(test_path).ok();
    }
}

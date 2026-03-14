use crate::block::Block;
use crate::blockchain::Blockchain;
use crate::transaction::{Transaction, TransactionPool};
use serde::{Deserialize, Serialize};
use std::fs;

pub struct Store;

#[derive(Serialize, Deserialize)]
struct BlockchainFile {
    chain: Vec<Block>,
    #[serde(default)]
    pending_transactions: Vec<Transaction>,
}

impl Store {
    /// Save blockchain to disk (JSON format)
    pub fn save_blockchain(blockchain: &Blockchain, path: &str) -> Result<(), String> {
        let file = BlockchainFile {
            chain: blockchain.chain.clone(),
            pending_transactions: blockchain
                .transaction_pool
                .pending_transactions()
                .to_vec(),
        };
        let json = serde_json::to_string_pretty(&file)
            .map_err(|e| format!("Serialization error: {}", e))?;

        fs::write(path, json).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Load blockchain from disk
    pub fn load_blockchain(path: &str) -> Result<Blockchain, String> {
        let json = fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

        let file: BlockchainFile =
            serde_json::from_str(&json).map_err(|e| format!("Deserialization error: {}", e))?;

        Ok(Blockchain {
            chain: file.chain,
            difficulty: 2,
            transaction_pool: TransactionPool::from_transactions(file.pending_transactions),
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

    #[test]
    fn test_save_load_with_pending_transactions() {
        use crate::wallet::Wallet;

        let mut blockchain = Blockchain::new();
        let wallet = Wallet::new();
        let mut tx = Transaction::new(wallet.address().to_string(), "bob".to_string(), 5.0);
        tx.sign(&wallet);
        blockchain.add_transaction(tx).unwrap();

        let test_path = "/tmp/test_blockchain_pending.json";
        Store::save_blockchain(&blockchain, test_path).unwrap();

        let loaded = Store::load_blockchain(test_path).unwrap();
        assert_eq!(loaded.transaction_pool.pending_count(), 1);

        fs::remove_file(test_path).ok();
    }
}

use serde::{Serialize, Deserialize};
use crate::transaction::Transaction;
use crate::crypto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub index: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub merkle_root: String,
    pub nonce: u64,
    pub difficulty: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub hash: String,
}

impl Block {
    /// Create a new block (header only, not yet mined)
    pub fn new(
        index: u64,
        previous_hash: String,
        transactions: Vec<Transaction>,
        difficulty: u32,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let merkle_root = Self::compute_merkle_root(&transactions);

        let header = BlockHeader {
            index,
            timestamp,
            previous_hash,
            merkle_root,
            nonce: 0,
            difficulty,
        };

        Block {
            header,
            transactions,
            hash: String::new(),
        }
    }

    /// Mine this block: increment nonce until PoW is satisfied
    pub fn mine(&mut self) -> u64 {
        let mut nonce = 0u64;
        loop {
            self.header.nonce = nonce;
            self.hash = self.compute_hash();

            if crypto::check_pow(&self.hash, self.header.difficulty) {
                return nonce;
            }

            nonce += 1;
        }
    }

    /// Compute block hash from header
    pub fn compute_hash(&self) -> String {
        let header_json = serde_json::to_string(&self.header)
            .unwrap_or_default();
        crypto::hash_sha256(header_json.as_bytes())
    }

    /// Verify this block's PoW is valid
    pub fn verify_pow(&self) -> bool {
        crypto::check_pow(&self.hash, self.header.difficulty)
    }

    /// Compute Merkle root (simplified: hash all tx IDs concatenated)
    fn compute_merkle_root(transactions: &[Transaction]) -> String {
        if transactions.is_empty() {
            return crypto::hash_sha256(b"empty");
        }

        let tx_ids: String = transactions
            .iter()
            .map(|tx| tx.id.clone())
            .collect::<Vec<_>>()
            .join("");

        crypto::hash_sha256(tx_ids.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = Block::new(0, "genesis".to_string(), vec![], 1);
        assert_eq!(block.header.index, 0);
        assert_eq!(block.header.previous_hash, "genesis");
    }

    #[test]
    fn test_block_mining() {
        let mut block = Block::new(0, "genesis".to_string(), vec![], 2);
        let nonce = block.mine();
        assert!(block.verify_pow());
        assert_eq!(block.header.nonce, nonce);
    }
}

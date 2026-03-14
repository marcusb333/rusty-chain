use crate::block::Block;
use crate::transaction::{Transaction, TransactionPool};

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: u32,
    pub transaction_pool: TransactionPool,
}

pub struct BlockchainStats {
    pub total_blocks: usize,
    pub total_transactions: usize,
    pub pending_transactions: usize,
    pub difficulty: u32,
    pub latest_hash: String,
}

impl Blockchain {
    pub fn new() -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            difficulty: 2,
            transaction_pool: TransactionPool::new(),
        };

        // Create and add genesis block
        let genesis = blockchain.create_genesis_block();
        blockchain.chain.push(genesis);

        blockchain
    }

    /// Create the first block in the chain
    fn create_genesis_block(&self) -> Block {
        let mut block = Block::new(0, "0".to_string(), vec![], self.difficulty);
        block.mine();
        block
    }

    /// Get the latest block in the chain
    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().expect("Chain should never be empty")
    }

    /// Add a pending transaction to the pool
    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        self.transaction_pool.add_transaction(tx)
    }

    /// Mine a new block with pending transactions
    pub fn mine_block(&mut self, miner_address: &str) -> Result<Block, String> {
        let latest = self.get_latest_block();
        let previous_hash = latest.hash.clone();
        let index = self.chain.len() as u64;

        // Take up to 10 pending transactions
        let transactions = self.transaction_pool.take_transactions(10);

        // Add coinbase transaction (mining reward)
        let mut block_txs = vec![Transaction::new(
            "system".to_string(),
            miner_address.to_string(),
            50.0,
        )];
        block_txs.extend(transactions);

        // Create and mine the block
        let mut block = Block::new(index, previous_hash, block_txs, self.difficulty);
        println!(
            "Mining block {} (difficulty: {})...",
            index, self.difficulty
        );

        let nonce = block.mine();
        println!("✓ Block mined! Nonce: {}, Hash: {}", nonce, block.hash);

        self.chain.push(block.clone());
        Ok(block)
    }

    /// Validate the entire chain
    pub fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            // Check PoW
            if !current.verify_pow() {
                println!("❌ Block {} failed PoW check", i);
                return false;
            }

            // Check chain linkage
            if current.header.previous_hash != previous.hash {
                println!("❌ Block {} has invalid previous_hash", i);
                return false;
            }

            // Check index consistency
            if current.header.index != previous.header.index + 1 {
                println!("❌ Block {} has invalid index", i);
                return false;
            }
        }

        println!("✓ Chain is valid!");
        true
    }

    /// Get chain statistics
    pub fn get_stats(&self) -> BlockchainStats {
        BlockchainStats {
            total_blocks: self.chain.len(),
            total_transactions: self.chain.iter().map(|b| b.transactions.len()).sum(),
            pending_transactions: self.transaction_pool.pending_count(),
            difficulty: self.difficulty,
            latest_hash: self.get_latest_block().hash.clone(),
        }
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::Wallet;

    fn signed_tx(from_wallet: &Wallet, to_address: &str, amount: f64) -> Transaction {
        let mut tx = Transaction::new(
            from_wallet.address().to_string(),
            to_address.to_string(),
            amount,
        );
        tx.sign(from_wallet);
        tx
    }

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].header.index, 0);
    }

    #[test]
    fn test_add_transaction() {
        let mut blockchain = Blockchain::new();
        let wallet = Wallet::new();
        let tx = signed_tx(&wallet, "bob", 10.0);
        blockchain.add_transaction(tx).unwrap();
        assert_eq!(blockchain.transaction_pool.pending_count(), 1);
    }

    #[test]
    fn test_mine_block() {
        let mut blockchain = Blockchain::new();
        let wallet = Wallet::new();
        blockchain
            .add_transaction(signed_tx(&wallet, "bob", 10.0))
            .unwrap();

        blockchain.mine_block("miner1").unwrap();
        assert_eq!(blockchain.chain.len(), 2);
    }

    #[test]
    fn test_chain_validation() {
        let mut blockchain = Blockchain::new();
        let wallet = Wallet::new();
        blockchain
            .add_transaction(signed_tx(&wallet, "bob", 10.0))
            .unwrap();
        blockchain.mine_block("miner1").unwrap();
        assert!(blockchain.is_valid());
    }

    #[test]
    fn test_genesis_block_has_valid_pow() {
        let blockchain = Blockchain::new();
        assert!(blockchain.chain[0].verify_pow());
    }

    #[test]
    fn test_get_latest_block_is_genesis_initially() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.get_latest_block().header.index, 0);
    }

    #[test]
    fn test_get_latest_block_updates_after_mining() {
        let mut blockchain = Blockchain::new();
        blockchain.mine_block("miner1").unwrap();
        assert_eq!(blockchain.get_latest_block().header.index, 1);
    }

    #[test]
    fn test_unsigned_transaction_rejected() {
        let mut blockchain = Blockchain::new();
        let tx = crate::transaction::Transaction::new("alice".to_string(), "bob".to_string(), 5.0);
        assert!(blockchain.add_transaction(tx).is_err());
    }

    #[test]
    fn test_transaction_pool_cleared_after_mining() {
        let mut blockchain = Blockchain::new();
        let wallet = Wallet::new();
        blockchain
            .add_transaction(signed_tx(&wallet, "bob", 10.0))
            .unwrap();
        assert_eq!(blockchain.transaction_pool.pending_count(), 1);

        blockchain.mine_block("miner1").unwrap();
        assert_eq!(blockchain.transaction_pool.pending_count(), 0);
    }

    #[test]
    fn test_tampered_chain_is_invalid() {
        let mut blockchain = Blockchain::new();
        blockchain.mine_block("miner1").unwrap();

        // Tamper with the genesis block hash to break chain linkage
        blockchain.chain[0].hash = "tampered_hash".to_string();
        assert!(!blockchain.is_valid());
    }

    #[test]
    fn test_get_stats_after_mining() {
        let mut blockchain = Blockchain::new();
        blockchain.mine_block("miner1").unwrap();

        let stats = blockchain.get_stats();
        assert_eq!(stats.total_blocks, 2);
        assert!(stats.total_transactions >= 1); // at least the coinbase
        assert_eq!(stats.pending_transactions, 0);
        assert_eq!(stats.difficulty, 2);
    }

    #[test]
    fn test_blocks_are_sequentially_indexed() {
        let mut blockchain = Blockchain::new();
        blockchain.mine_block("miner1").unwrap();
        blockchain.mine_block("miner1").unwrap();

        for (i, block) in blockchain.chain.iter().enumerate() {
            assert_eq!(block.header.index, i as u64);
        }
    }
}

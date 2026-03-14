use crate::crypto;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
    pub id: String,
}

impl Transaction {
    pub fn new(from: String, to: String, amount: f64) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let tx = Transaction {
            from,
            to,
            amount,
            timestamp,
            id: String::new(),
        };

        // Hash the transaction data for an ID
        let id =
            crypto::hash_object(&(&tx.from, &tx.to, tx.amount, tx.timestamp)).unwrap_or_default();

        Transaction { id, ..tx }
    }

    pub fn is_valid(&self) -> bool {
        // TODO: Verify signature in real implementation
        self.amount > 0.0 && !self.from.is_empty() && !self.to.is_empty()
    }
}

pub struct TransactionPool {
    pending: Vec<Transaction>,
}

impl TransactionPool {
    pub fn new() -> Self {
        TransactionPool {
            pending: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) -> Result<(), String> {
        if !tx.is_valid() {
            return Err("Invalid transaction".to_string());
        }
        self.pending.push(tx);
        Ok(())
    }

    pub fn take_transactions(&mut self, count: usize) -> Vec<Transaction> {
        self.pending
            .drain(..count.min(self.pending.len()))
            .collect()
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for TransactionPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let tx = Transaction::new("alice".to_string(), "bob".to_string(), 10.5);
        assert_eq!(tx.from, "alice");
        assert_eq!(tx.to, "bob");
        assert_eq!(tx.amount, 10.5);
        assert!(!tx.id.is_empty());
    }

    #[test]
    fn test_transaction_validity() {
        let valid_tx = Transaction::new("alice".to_string(), "bob".to_string(), 10.0);
        assert!(valid_tx.is_valid());

        let mut invalid_tx = Transaction::new("alice".to_string(), "bob".to_string(), 10.0);
        invalid_tx.amount = -5.0;
        assert!(!invalid_tx.is_valid());
    }

    #[test]
    fn test_transaction_pool() {
        let mut pool = TransactionPool::new();
        let tx = Transaction::new("alice".to_string(), "bob".to_string(), 10.0);

        pool.add_transaction(tx).unwrap();
        assert_eq!(pool.pending_count(), 1);

        let txs = pool.take_transactions(1);
        assert_eq!(txs.len(), 1);
        assert_eq!(pool.pending_count(), 0);
    }
}

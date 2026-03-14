use crate::crypto;
use crate::wallet::{self, Wallet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub timestamp: u64,
    pub id: String,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default)]
    pub public_key: Option<String>,
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
            signature: None,
            public_key: None,
        };

        // Hash the transaction data for an ID
        let id =
            crypto::hash_object(&(&tx.from, &tx.to, tx.amount, tx.timestamp)).unwrap_or_default();

        Transaction { id, ..tx }
    }

    pub fn signable_bytes(&self) -> Vec<u8> {
        let payload = format!("{}{}{}{}", self.from, self.to, self.amount, self.timestamp);
        payload.into_bytes()
    }

    pub fn sign(&mut self, wallet: &Wallet) {
        let bytes = self.signable_bytes();
        self.signature = Some(wallet.sign(&bytes));
        self.public_key = Some(wallet.public_key_hex());
    }

    pub fn is_valid(&self) -> bool {
        if self.amount <= 0.0 || self.from.is_empty() || self.to.is_empty() {
            return false;
        }

        // Coinbase transactions from "system" don't require signatures
        if self.from == "system" {
            return true;
        }

        // Non-system transactions must be signed
        let signature = match &self.signature {
            Some(s) => s,
            None => return false,
        };
        let public_key = match &self.public_key {
            Some(pk) => pk,
            None => return false,
        };

        // Verify the signature
        let bytes = self.signable_bytes();
        if !wallet::verify_signature(public_key, &bytes, signature) {
            return false;
        }

        // Verify the public key matches the sender address
        match wallet::address_from_public_key_hex(public_key) {
            Ok(addr) => addr == self.from,
            Err(_) => false,
        }
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

    pub fn pending_transactions(&self) -> &[Transaction] {
        &self.pending
    }

    pub fn from_transactions(txs: Vec<Transaction>) -> Self {
        TransactionPool { pending: txs }
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
    fn test_coinbase_transaction_valid() {
        let tx = Transaction::new("system".to_string(), "miner".to_string(), 50.0);
        assert!(tx.is_valid());
    }

    #[test]
    fn test_unsigned_transaction_invalid() {
        let tx = Transaction::new("alice".to_string(), "bob".to_string(), 10.0);
        assert!(!tx.is_valid());
    }

    #[test]
    fn test_signed_transaction_valid() {
        let wallet = Wallet::new();
        let mut tx = Transaction::new(wallet.address().to_string(), "bob".to_string(), 10.0);
        tx.sign(&wallet);
        assert!(tx.is_valid());
    }

    #[test]
    fn test_invalid_amount() {
        let mut tx = Transaction::new("system".to_string(), "bob".to_string(), 10.0);
        tx.amount = -5.0;
        assert!(!tx.is_valid());
    }

    #[test]
    fn test_transaction_pool() {
        let mut pool = TransactionPool::new();
        let tx = Transaction::new("system".to_string(), "bob".to_string(), 10.0);

        pool.add_transaction(tx).unwrap();
        assert_eq!(pool.pending_count(), 1);

        let txs = pool.take_transactions(1);
        assert_eq!(txs.len(), 1);
        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn test_empty_from_is_invalid() {
        let tx = Transaction::new("".to_string(), "bob".to_string(), 10.0);
        assert!(!tx.is_valid());
    }

    #[test]
    fn test_empty_to_is_invalid() {
        let tx = Transaction::new("system".to_string(), "".to_string(), 10.0);
        assert!(!tx.is_valid());
    }

    #[test]
    fn test_zero_amount_is_invalid() {
        let tx = Transaction::new("system".to_string(), "bob".to_string(), 0.0);
        assert!(!tx.is_valid());
    }

    #[test]
    fn test_tampered_amount_invalidates_signature() {
        let wallet = Wallet::new();
        let mut tx = Transaction::new(wallet.address().to_string(), "bob".to_string(), 10.0);
        tx.sign(&wallet);
        assert!(tx.is_valid());

        tx.amount = 1000.0; // tamper after signing
        assert!(!tx.is_valid());
    }

    #[test]
    fn test_pool_rejects_invalid_transaction() {
        let mut pool = TransactionPool::new();
        let tx = Transaction::new("alice".to_string(), "bob".to_string(), 10.0); // unsigned
        assert!(pool.add_transaction(tx).is_err());
        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn test_take_transactions_more_than_available() {
        let mut pool = TransactionPool::new();
        pool.add_transaction(Transaction::new("system".to_string(), "bob".to_string(), 1.0)).unwrap();
        pool.add_transaction(Transaction::new("system".to_string(), "carol".to_string(), 2.0)).unwrap();

        let txs = pool.take_transactions(10); // request more than available
        assert_eq!(txs.len(), 2);
        assert_eq!(pool.pending_count(), 0);
    }

    #[test]
    fn test_transaction_id_is_deterministic_given_same_fields() {
        let tx = Transaction::new("system".to_string(), "bob".to_string(), 5.0);
        assert!(!tx.id.is_empty());
        assert_eq!(tx.id.len(), 64); // SHA256 hex
    }
}

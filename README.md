# Rust PoW Blockchain

A clean, modular Proof-of-Work blockchain implementation in Rust for learning and experimentation.

## Architecture

```
blockchain/
├── src/
│   ├── main.rs          # Demo application
│   ├── crypto.rs        # SHA256 hashing & PoW validation
│   ├── transaction.rs   # Transaction & TransactionPool
│   ├── block.rs         # Block structure & mining
│   ├── blockchain.rs    # Main chain logic & validation
│   └── persistence.rs   # Save/load blockchain to disk
└── Cargo.toml           # Dependencies
```

### Module Breakdown

**`crypto`**
- `hash_sha256()` - Raw SHA256 hashing
- `hash_object()` - Serialize struct and hash it
- `check_pow()` - Validate PoW (leading zeros)

**`transaction`**
- `Transaction` - From, to, amount, timestamp, ID
- `TransactionPool` - Queue of pending transactions
- Validation (checks amount > 0, non-empty addresses)

**`block`**
- `BlockHeader` - Index, timestamp, prev_hash, merkle_root, nonce, difficulty
- `Block` - Header + transactions + computed hash
- `mine()` - Increment nonce until PoW satisfied
- `verify_pow()` - Check block meets difficulty target
- `compute_merkle_root()` - Hash of all transaction IDs

**`blockchain`**
- `Blockchain` - Chain vector + difficulty + transaction pool
- `mine_block()` - Create block from pending txs + coinbase
- `is_valid()` - Verify PoW + chain linkage for all blocks
- `get_stats()` - Chain statistics
- Genesis block created automatically

**`persistence`**
- `Store::save_blockchain()` - Write chain to JSON
- `Store::load_blockchain()` - Load chain from JSON
- `Store::print_blockchain()` - Pretty-print chain details

## Running the Demo

```bash
cd blockchain
cargo run
```

Output:
- ✓ Genesis block mined
- ✓ Two rounds of transactions mined
- ✓ Chain validation
- ✓ Blockchain saved to `blockchain.json`
- ✓ Blockchain reloaded from disk

## Running Tests

```bash
cargo test
```

All 12 unit tests pass:
- Crypto hashing & PoW validation
- Transaction creation & pool management
- Block mining & validation
- Blockchain operations
- Persistence (save/load)

## Current Features

✓ SHA256 hashing
✓ Proof-of-Work mining (adjustable difficulty)
✓ Block structure with merkle roots
✓ Transaction pool management
✓ Coinbase rewards for mining
✓ Full chain validation
✓ Persistence to JSON
✓ Comprehensive test coverage

## Next Steps (Easy → Hard)

### 1. Difficulty Adjustment
Implement dynamic difficulty based on block time:
```rust
// In blockchain.rs
pub fn adjust_difficulty(&mut self) {
    let recent_blocks = &self.chain[self.chain.len() - 10..];
    let avg_time = /* compute average block time */;
    if avg_time > TARGET_BLOCK_TIME {
        self.difficulty -= 1;
    } else if avg_time < TARGET_BLOCK_TIME {
        self.difficulty += 1;
    }
}
```

### 2. Digital Signatures
Add transaction signing with `secp256k1`:
```toml
# In Cargo.toml
secp256k1 = "0.27"
```

Then modify `Transaction` to include signature field and verify before adding to pool.

### 3. State Machine
Track account balances:
```rust
// In blockchain.rs
pub balance_map: HashMap<String, f64>
```

Validate transaction outputs before mining.

### 4. Persistence Upgrade
Replace JSON with `rocksdb`:
```toml
rocksdb = "0.19"
```

Much faster for large chains.

### 5. P2P Networking
Add `tokio` + `libp2p` for multi-node sync:
```toml
tokio = { version = "1", features = ["full"] }
libp2p = "0.51"
```

Enable nodes to broadcast blocks and sync.

### 6. Fork Resolution
Implement longest-chain rule:
```rust
pub fn resolve_fork(&mut self, competing_chain: Vec<Block>) {
    if competing_chain.len() > self.chain.len() {
        self.chain = competing_chain;
    }
}
```

### 7. Smart Contracts
Simple VM for executing contract code on transactions.

### 8. Merkle Proof / SPV
Implement merkle tree proof for light clients.

## Performance

- **Mining time** (difficulty 2): ~300-400 hash iterations
- **Chain validation**: O(n) linear scan
- **Block serialization**: JSON (can upgrade to bincode for speed)

## Design Notes

- **Simplified merkle tree**: Concatenates all tx IDs and hashes once (real chains use binary tree)
- **No UTXO model**: Uses simple from/to/amount (real chains use UTXOs)
- **In-memory pool**: Transactions lost on restart (real chains persist)
- **Single difficulty**: Not adjusted dynamically (step 1 in next steps)
- **No consensus rules**: Accepts any block with valid PoW (need tx validation, balance checks)

## Educational Value

This codebase is great for learning:
- How blockchain immutability works (hash chain)
- Why PoW creates security (computational cost)
- Block structure & merkle trees
- Mining & nonce increment loops
- Chain validation & state consistency
- Modularity in Rust (separation of concerns)
- Unit testing in Rust
- Serialization with serde
- Ownership & lifetimes (minimal in this design, but present)

## License

MIT (do what you want)

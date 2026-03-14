# rusty-chain

A modular Proof-of-Work blockchain in Rust with secp256k1 wallets and a CLI.

## Architecture

```
src/
├── main.rs          # CLI entry point (clap)
├── lib.rs           # Public module exports
├── crypto.rs        # SHA256 hashing & PoW validation
├── wallet.rs        # secp256k1 key pairs, signing, address derivation
├── transaction.rs   # Transaction + TransactionPool
├── block.rs         # Block structure & PoW mining
├── blockchain.rs    # Chain logic & validation
└── persistence.rs   # JSON save/load
```

### Data Flow

```
CLI (main.rs)
 ├── wallet create/list/info  ──►  WalletStore (wallet.rs)
 │                                   └── ~/.rusty-chain/wallets.json
 │
 ├── send --from --to --amount
 │    ├── load Wallet (wallet.rs)
 │    ├── sign Transaction (transaction.rs)
 │    └── add to Blockchain pool (blockchain.rs)
 │
 ├── mine --miner
 │    ├── take pending txs from TransactionPool
 │    ├── prepend coinbase Transaction
 │    ├── mine Block (block.rs) — increment nonce until PoW met
 │    └── append to chain, save (persistence.rs)
 │
 └── status  ──►  Blockchain.get_stats()
```

### Module Breakdown

**`crypto`**
- `hash_sha256(data)` — raw SHA256 → 64-char hex string
- `hash_object(obj)` — serialize + hash any `Serialize` value
- `check_pow(hash, difficulty)` — true if hash has N leading zeros

**`wallet`**
- `Wallet` — secp256k1 key pair; derives a 40-char hex address from the public key
- `Wallet::sign(message)` — ECDSA signature (hex-encoded)
- `verify_signature(pk_hex, message, sig_hex)` — verify without a `Wallet` instance
- `address_from_public_key_hex(pk_hex)` — derive address from raw public key
- `WalletStore` — persist/load wallets as JSON in `~/.rusty-chain/wallets.json`

**`transaction`**
- `Transaction` — `from`, `to`, `amount`, `timestamp`, `id` (SHA256), `signature`, `public_key`
- `Transaction::sign(wallet)` — attach ECDSA signature
- `Transaction::is_valid()` — validates amount > 0, non-empty addresses, and signature (skipped for `from = "system"` coinbase)
- `TransactionPool` — pending queue; rejects invalid transactions on insert

**`block`**
- `BlockHeader` — `index`, `timestamp`, `previous_hash`, `merkle_root`, `nonce`, `difficulty`
- `Block::mine()` — increments nonce until `check_pow` passes; returns winning nonce
- `Block::verify_pow()` — checks stored hash meets difficulty target
- `Block::compute_merkle_root()` — SHA256 of all transaction IDs concatenated

**`blockchain`**
- `Blockchain` — `Vec<Block>` + `difficulty` + `TransactionPool`
- `Blockchain::new()` — creates genesis block (auto-mined at difficulty 2)
- `mine_block(miner_addr)` — drains up to 10 pending txs, prepends coinbase (50 coins), mines and appends
- `is_valid()` — verifies PoW, chain linkage (`previous_hash`), and index sequence for every block
- `get_stats()` — totals: blocks, transactions, pending, difficulty, latest hash

**`persistence`**
- `Store::save_blockchain(blockchain, path)` — serialize chain + pending pool to JSON
- `Store::load_blockchain(path)` — deserialize and restore
- `Store::print_blockchain(blockchain)` — summary to stdout

## CLI Usage

```bash
cargo build --release

# Wallet management
./target/release/rusty-chain wallet create alice
./target/release/rusty-chain wallet create bob
./target/release/rusty-chain wallet list
./target/release/rusty-chain wallet info alice

# Send coins (adds signed transaction to pending pool)
./target/release/rusty-chain send --from alice --to bob --amount 10.5

# Mine pending transactions (earns 50-coin coinbase reward)
./target/release/rusty-chain mine --miner alice

# Show chain status
./target/release/rusty-chain status

# Run the built-in demo
./target/release/rusty-chain demo
```

## Running Tests

```bash
cargo test
```

55 unit tests across all modules:

| Module | Tests |
|--------|-------|
| `crypto` | hashing correctness, PoW boundary cases, `hash_object` |
| `wallet` | key generation, sign/verify, address derivation, error handling |
| `transaction` | validation rules, signature tampering, pool operations |
| `block` | mining, PoW verification, hash determinism, merkle root |
| `blockchain` | chain creation, validation, tamper detection, stats |
| `persistence` | save/load round-trip with pending transactions |

## Current Features

- SHA256 hashing with `sha2`
- Proof-of-Work mining (adjustable difficulty, leading-zero target)
- secp256k1 ECDSA wallet key pairs and transaction signing
- Block structure with simplified Merkle root
- Transaction pool with signature validation
- Coinbase rewards (50 coins per mined block)
- Full chain validation (PoW + linkage + index)
- JSON persistence to `~/.rusty-chain/`
- CLI with wallet management, send, mine, and status commands

## Design Notes

- **Simplified Merkle tree** — concatenates all tx IDs and hashes once; real chains use a binary tree
- **No UTXO model** — simple from/to/amount; no balance tracking or double-spend prevention
- **In-memory pool** — persisted to JSON on every write, loaded on startup
- **Fixed difficulty** — not adjusted dynamically
- **No p2p** — single-node only

## Potential Next Steps

1. **Dynamic difficulty** — adjust based on average block time over last N blocks
2. **Balance tracking** — maintain a `HashMap<address, f64>` and reject overspend
3. **UTXO model** — replace balances with unspent transaction outputs
4. **RocksDB persistence** — replace JSON for performance at scale
5. **P2P networking** — `tokio` + `libp2p` for multi-node sync and block broadcast
6. **Fork resolution** — longest-chain rule when competing chains are received
7. **Merkle proofs** — proper binary Merkle tree for SPV light clients
8. **Simple VM** — scripting layer for basic smart contracts

## License

MIT

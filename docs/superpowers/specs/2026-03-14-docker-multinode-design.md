# Docker Multi-Node Simulation Design

**Date:** 2026-03-14
**Status:** Approved

## Overview

Add a `node` subcommand to the `rusty-chain` binary that runs as a persistent process participating in a simulated blockchain network. Multiple nodes communicate via NATS message broker, gossip transactions and blocks, mine autonomously, and keep local chain and wallet state synchronized. Orchestrated with Docker Compose.

## Architecture

Each container runs one `rusty-chain node --name <name>` process.

### Startup Sequence

1. Connect to NATS (`NATS_URL` env var; falls back to `--nats-url` CLI arg)
2. Auto-create a local wallet named `$NODE_NAME` if none exists
3. Publish own public wallet data to `wallets.sync` (name + address + public_key only — never secret_key)
4. Collect `wallets.sync` responses for 2s; merge into local store (union by address, ignore conflicts)
5. Send NATS request on `chain.sync` inbox (NATS native request/reply); collect responses for 3s; accept the longest valid chain received
6. Reconcile pending transaction pool: drop any transactions already included in the accepted chain
7. Enter main async loop

### Main Loop (concurrent tokio tasks)

| Task | Behavior |
|---|---|
| **Subscriber** | Listens on `blocks.new` and `txns.new`; deduplicates by block hash / tx id before applying; validates then saves to local state |
| **Publisher** | When this node mines a block or creates a transaction, publishes to the relevant subject |
| **Activity** | Every `TX_INTERVAL_MIN`–`TX_INTERVAL_MAX` seconds (randomized), picks two known wallets at random and creates a signed transaction; publishes to `txns.new`; mines via `spawn_blocking` when pending pool reaches `MINE_THRESHOLD` |

### Deduplication

Each node maintains in-memory `HashSet<String>` for seen block hashes and transaction IDs. Incoming messages whose hash/id is already in the set are silently dropped before validation.

### NATS Subjects & Wire Formats

All payloads are JSON-serialized.

| Subject | Direction | Payload type | Notes |
|---|---|---|---|
| `chain.sync` | request/reply | `Vec<Block>` | NATS inbox pattern; reply contains the responder's full chain (excluding pending pool) |
| `blocks.new` | broadcast | `Block` | Full block struct |
| `txns.new` | broadcast | `Transaction` | Full transaction struct |
| `wallets.sync` | broadcast | `WalletPublicData` | New struct: `{ name, address, public_key }` — never includes secret_key |

### Async Runtime

`fn main()` is annotated `#[tokio::main]`. The `Node` subcommand arm calls `node::run(...).await`. All other subcommand arms are synchronous and unaffected.

### Mining in Async Context

`Block::mine()` is CPU-bound. The activity task calls `tokio::task::spawn_blocking(|| blockchain.mine_block(addr))` to avoid blocking the executor. The result is awaited and then published on `blocks.new`.

## Components

### New: `src/node.rs`

Async node runtime. Owns NATS connection, seen-hashes/ids dedup sets, tokio task spawning, and the activity loop. Public interface: `pub async fn run(name, nats_url, mine_threshold, tx_interval_min, tx_interval_max) -> Result<(), String>`.

Error handling: NATS disconnect → log error and exit with non-zero code (Docker will restart if `restart: unless-stopped`). Invalid incoming block/tx → log and drop. Mining failure → log and continue loop.

### New: `src/wallet.rs` addition — `WalletPublicData`

```rust
#[derive(Serialize, Deserialize)]
pub struct WalletPublicData {
    pub name: String,
    pub address: String,
    pub public_key: String,
}
```

### Updated: `src/main.rs`

Add `Commands::Node` subcommand:

```
node --name <name> [--nats-url <url>] [--mine-threshold <n>] [--tx-interval-min <s>] [--tx-interval-max <s>]
```

Reads `NODE_NAME`, `NATS_URL`, `MINE_THRESHOLD`, `TX_INTERVAL_MIN`, `TX_INTERVAL_MAX` env vars via clap `env` attribute; CLI args take precedence.

### New: `Dockerfile`

Multi-stage build:
- Stage 1: `rust:1-alpine` — `cargo build --release`
- Stage 2: `alpine:3` — copy binary to `/usr/local/bin/rusty-chain`
- `CMD ["rusty-chain", "node"]` — `NODE_NAME` and `NATS_URL` supplied via env

The container runs as root; data path is `/root/.rusty-chain/`.

### New: `docker-compose.yml`

```yaml
services:
  nats:
    image: nats:latest
    ports: ["4222:4222"]
    healthcheck:
      test: ["CMD", "nats-server", "--help"]
      interval: 5s
      timeout: 3s
      retries: 5

  node-alice:
    build: .
    environment:
      NODE_NAME: alice
      NATS_URL: nats://nats:4222
    volumes:
      - alice-data:/root/.rusty-chain
    depends_on:
      nats:
        condition: service_healthy
    restart: unless-stopped

  node-bob:   # same pattern, NODE_NAME: bob, volume: bob-data
  node-charlie: # same pattern, NODE_NAME: charlie, volume: charlie-data

volumes:
  alice-data:
  bob-data:
  charlie-data:
```

### New: `docker-compose.test.yml`

Adds a `test-runner` service that mounts the source tree and runs `cargo test --test '*'`. The integration tests use `testcontainers` to manage their own NATS instance — this compose file only adds the runner container; it does NOT re-use the main NATS service to avoid port conflicts.

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `NODE_NAME` | required | Wallet name for this node |
| `NATS_URL` | `nats://nats:4222` | NATS server address |
| `MINE_THRESHOLD` | `3` | Pending txns required before mining |
| `TX_INTERVAL_MIN` | `5` | Minimum seconds between activity loop ticks |
| `TX_INTERVAL_MAX` | `15` | Maximum seconds between activity loop ticks |

## Testing

### Unit Tests

Existing `#[cfg(test)]` blocks stay inline in each module. No changes.

### Integration Tests

New `tests/integration/` directory. Each file is a separate `cargo test` integration test target.

```
tests/
  integration/
    node_sync.rs      # two nodes sync chain on startup via chain.sync
    gossip_blocks.rs  # mined block on blocks.new propagates to all nodes
    gossip_txns.rs    # transaction on txns.new received and deduplicated
    wallet_sync.rs    # WalletPublicData merges correctly, no secret_key leakage
```

All use `testcontainers` crate to spin up a real NATS container in-process.

Run with: `cargo test --test '*'`

## File Changes Summary

| Action | File |
|---|---|
| Add | `src/node.rs` |
| Modify | `src/main.rs` — add Node subcommand, `#[tokio::main]` |
| Modify | `src/wallet.rs` — add `WalletPublicData` struct |
| Modify | `Cargo.toml` — add `tokio`, `async-nats`, `testcontainers` (dev) |
| Add | `Dockerfile` |
| Add | `docker-compose.yml` |
| Add | `docker-compose.test.yml` |
| Add | `tests/integration/node_sync.rs` |
| Add | `tests/integration/gossip_blocks.rs` |
| Add | `tests/integration/gossip_txns.rs` |
| Add | `tests/integration/wallet_sync.rs` |

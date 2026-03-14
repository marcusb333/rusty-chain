use blockchain::{Blockchain, Store, Transaction};

fn main() {
    println!("🔗 Rust PoW Blockchain - Starter Architecture\n");

    // Initialize blockchain
    let mut blockchain = Blockchain::new();
    println!("✓ Genesis block created\n");

    // === Round 1: Add and mine transactions ===
    println!("📝 Round 1: Adding transactions...");
    blockchain
        .add_transaction(Transaction::new(
            "alice".to_string(),
            "bob".to_string(),
            10.5,
        ))
        .expect("Failed to add transaction");

    blockchain
        .add_transaction(Transaction::new(
            "bob".to_string(),
            "charlie".to_string(),
            5.0,
        ))
        .expect("Failed to add transaction");

    println!("✓ 2 transactions added\n");

    // Mine block 1
    println!("⛏️  Mining block 1...");
    blockchain
        .mine_block("miner1")
        .expect("Failed to mine block");

    println!();

    // === Round 2: More transactions ===
    println!("📝 Round 2: Adding more transactions...");
    blockchain
        .add_transaction(Transaction::new(
            "charlie".to_string(),
            "alice".to_string(),
            3.5,
        ))
        .expect("Failed to add transaction");

    blockchain
        .add_transaction(Transaction::new(
            "alice".to_string(),
            "dave".to_string(),
            2.0,
        ))
        .expect("Failed to add transaction");

    println!("✓ 2 transactions added\n");

    // Mine block 2
    println!("⛏️  Mining block 2...");
    blockchain
        .mine_block("miner1")
        .expect("Failed to mine block");

    println!();

    // === Validation ===
    println!("🔍 Validating blockchain...");
    blockchain.is_valid();

    println!();

    // === Statistics ===
    let stats = blockchain.get_stats();
    println!("📊 Blockchain Stats:");
    println!("   Blocks:        {}", stats.total_blocks);
    println!("   Transactions:  {}", stats.total_transactions);
    println!("   Pending:       {}", stats.pending_transactions);
    println!("   Difficulty:    {}", stats.difficulty);
    println!("   Latest hash:   {}", &stats.latest_hash[..16]);

    // === Print chain details ===
    Store::print_blockchain(&blockchain);

    // === Persistence ===
    println!("\n💾 Saving blockchain...");
    Store::save_blockchain(&blockchain, "blockchain.json").expect("Failed to save");
    println!("✓ Saved to blockchain.json");

    // === Load and verify ===
    println!("\n📂 Loading blockchain from disk...");
    let loaded = Store::load_blockchain("blockchain.json").expect("Failed to load");
    println!("✓ Loaded {} blocks from disk", loaded.chain.len());
}

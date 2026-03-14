use blockchain::{Blockchain, Store, Transaction, Wallet};

fn main() {
    println!("🔗 Rust PoW Blockchain - Starter Architecture\n");

    // Create wallets
    let alice = Wallet::new();
    let bob = Wallet::new();
    let miner = Wallet::new();

    println!("👛 Wallets created:");
    println!("   Alice:  {}", alice.address());
    println!("   Bob:    {}", bob.address());
    println!("   Miner:  {}\n", miner.address());

    // Initialize blockchain
    let mut blockchain = Blockchain::new();
    println!("✓ Genesis block created\n");

    // === Round 1: Add and mine transactions ===
    println!("📝 Round 1: Adding transactions...");

    let mut tx1 = Transaction::new(alice.address().to_string(), bob.address().to_string(), 10.5);
    tx1.sign(&alice);
    blockchain
        .add_transaction(tx1)
        .expect("Failed to add transaction");

    let mut tx2 = Transaction::new(bob.address().to_string(), alice.address().to_string(), 5.0);
    tx2.sign(&bob);
    blockchain
        .add_transaction(tx2)
        .expect("Failed to add transaction");

    println!("✓ 2 transactions added\n");

    // Mine block 1
    println!("⛏️  Mining block 1...");
    blockchain
        .mine_block(miner.address())
        .expect("Failed to mine block");

    println!();

    // === Round 2: More transactions ===
    println!("📝 Round 2: Adding more transactions...");

    let mut tx3 = Transaction::new(bob.address().to_string(), alice.address().to_string(), 3.5);
    tx3.sign(&bob);
    blockchain
        .add_transaction(tx3)
        .expect("Failed to add transaction");

    let mut tx4 = Transaction::new(alice.address().to_string(), bob.address().to_string(), 2.0);
    tx4.sign(&alice);
    blockchain
        .add_transaction(tx4)
        .expect("Failed to add transaction");

    println!("✓ 2 transactions added\n");

    // Mine block 2
    println!("⛏️  Mining block 2...");
    blockchain
        .mine_block(miner.address())
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

use blockchain::persistence::Store;
use blockchain::transaction::Transaction;
use blockchain::wallet::{Wallet, WalletStore};
use blockchain::Blockchain;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rusty-chain", about = "A simple PoW blockchain CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wallet management
    Wallet {
        #[command(subcommand)]
        action: WalletAction,
    },
    /// Send coins from one wallet to another
    Send {
        /// Sender wallet name
        #[arg(long)]
        from: String,
        /// Recipient address
        #[arg(long)]
        to: String,
        /// Amount to send
        #[arg(long)]
        amount: f64,
    },
    /// Mine a new block
    Mine {
        /// Miner wallet name
        #[arg(long)]
        miner: String,
    },
    /// Show blockchain status
    Status,
    /// Run the demo (original hardcoded flow)
    Demo,
}

#[derive(Subcommand)]
enum WalletAction {
    /// Create a new wallet
    Create {
        /// Name for the wallet
        name: String,
    },
    /// List all wallets
    List,
    /// Show details for a specific wallet
    Info {
        /// Wallet name
        name: String,
    },
}

fn load_or_create_blockchain() -> Blockchain {
    let path = WalletStore::blockchain_path();
    if path.exists() {
        Store::load_blockchain(path.to_str().unwrap()).unwrap_or_else(|_| Blockchain::new())
    } else {
        Blockchain::new()
    }
}

fn save_blockchain(blockchain: &Blockchain) {
    let dir = WalletStore::data_dir();
    std::fs::create_dir_all(&dir).expect("Failed to create data dir");
    let path = WalletStore::blockchain_path();
    Store::save_blockchain(blockchain, path.to_str().unwrap()).expect("Failed to save blockchain");
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Wallet { action } => match action {
            WalletAction::Create { name } => {
                let wallets = WalletStore::load_wallets();
                if wallets.iter().any(|w| w.name == name) {
                    eprintln!("Wallet '{}' already exists.", name);
                    std::process::exit(1);
                }

                let wallet = Wallet::new();
                let data = wallet.to_data(&name);

                let mut wallets = wallets;
                wallets.push(data);
                WalletStore::save_wallets(&wallets).expect("Failed to save wallets");

                println!("Wallet '{}' created.", name);
                println!("  Address: {}", wallet.address());
            }
            WalletAction::List => {
                let wallets = WalletStore::load_wallets();
                if wallets.is_empty() {
                    println!("No wallets found. Create one with: rusty-chain wallet create <name>");
                    return;
                }
                println!("{:<12} ADDRESS", "NAME");
                for w in &wallets {
                    println!("{:<12} {}", w.name, w.address);
                }
            }
            WalletAction::Info { name } => {
                let wallets = WalletStore::load_wallets();
                match wallets.iter().find(|w| w.name == name) {
                    Some(w) => {
                        println!("Name:       {}", w.name);
                        println!("Address:    {}", w.address);
                        println!("Public Key: {}", w.public_key);
                    }
                    None => {
                        eprintln!("Wallet '{}' not found.", name);
                        std::process::exit(1);
                    }
                }
            }
        },

        Commands::Send { from, to, amount } => {
            let wallet = WalletStore::find_wallet(&from).unwrap_or_else(|| {
                eprintln!("Wallet '{}' not found.", from);
                std::process::exit(1);
            });

            // Resolve "to" — could be a wallet name or raw address
            let to_address = {
                let wallets = WalletStore::load_wallets();
                wallets
                    .iter()
                    .find(|w| w.name == to)
                    .map(|w| w.address.clone())
                    .unwrap_or(to)
            };

            let mut tx = Transaction::new(wallet.address().to_string(), to_address.clone(), amount);
            tx.sign(&wallet);

            let mut blockchain = load_or_create_blockchain();
            blockchain
                .add_transaction(tx)
                .expect("Failed to add transaction");
            save_blockchain(&blockchain);

            println!(
                "Transaction added: {} -> {} ({} coins)",
                from, to_address, amount
            );
            println!(
                "Pending transactions: {}",
                blockchain.transaction_pool.pending_count()
            );
        }

        Commands::Mine { miner } => {
            let wallet = WalletStore::find_wallet(&miner).unwrap_or_else(|| {
                eprintln!("Wallet '{}' not found.", miner);
                std::process::exit(1);
            });

            let mut blockchain = load_or_create_blockchain();
            let pending = blockchain.transaction_pool.pending_count();

            if pending == 0 {
                println!("No pending transactions to mine.");
                return;
            }

            println!("Mining block with {} pending transaction(s)...", pending);
            let block = blockchain
                .mine_block(wallet.address())
                .expect("Failed to mine block");

            save_blockchain(&blockchain);

            println!(
                "Block #{} mined! Hash: {}...",
                block.header.index,
                &block.hash[..16]
            );
            println!("  Transactions: {}", block.transactions.len());
            println!("  Nonce: {}", block.header.nonce);
        }

        Commands::Status => {
            let blockchain = load_or_create_blockchain();
            let stats = blockchain.get_stats();

            println!("Blockchain Status:");
            println!("  Blocks:       {}", stats.total_blocks);
            println!("  Transactions: {}", stats.total_transactions);
            println!("  Pending:      {}", stats.pending_transactions);
            println!("  Difficulty:   {}", stats.difficulty);
            println!("  Latest hash:  {}...", &stats.latest_hash[..16]);

            Store::print_blockchain(&blockchain);
        }

        Commands::Demo => {
            run_demo();
        }
    }
}

fn run_demo() {
    println!("Rust PoW Blockchain - Demo\n");

    let alice = Wallet::new();
    let bob = Wallet::new();
    let miner = Wallet::new();

    println!("Wallets created:");
    println!("  Alice:  {}", alice.address());
    println!("  Bob:    {}", bob.address());
    println!("  Miner:  {}\n", miner.address());

    let mut blockchain = Blockchain::new();
    println!("Genesis block created.\n");

    // Round 1
    println!("Round 1: Adding transactions...");
    let mut tx1 = Transaction::new(alice.address().to_string(), bob.address().to_string(), 10.5);
    tx1.sign(&alice);
    blockchain.add_transaction(tx1).expect("Failed");

    let mut tx2 = Transaction::new(bob.address().to_string(), alice.address().to_string(), 5.0);
    tx2.sign(&bob);
    blockchain.add_transaction(tx2).expect("Failed");
    println!("2 transactions added.\n");

    println!("Mining block 1...");
    blockchain.mine_block(miner.address()).expect("Failed");
    println!();

    // Round 2
    println!("Round 2: Adding transactions...");
    let mut tx3 = Transaction::new(bob.address().to_string(), alice.address().to_string(), 3.5);
    tx3.sign(&bob);
    blockchain.add_transaction(tx3).expect("Failed");

    let mut tx4 = Transaction::new(alice.address().to_string(), bob.address().to_string(), 2.0);
    tx4.sign(&alice);
    blockchain.add_transaction(tx4).expect("Failed");
    println!("2 transactions added.\n");

    println!("Mining block 2...");
    blockchain.mine_block(miner.address()).expect("Failed");
    println!();

    println!("Validating blockchain...");
    blockchain.is_valid();
    println!();

    let stats = blockchain.get_stats();
    println!("Blockchain Stats:");
    println!("  Blocks:        {}", stats.total_blocks);
    println!("  Transactions:  {}", stats.total_transactions);
    println!("  Pending:       {}", stats.pending_transactions);
    println!("  Difficulty:    {}", stats.difficulty);
    println!("  Latest hash:   {}...", &stats.latest_hash[..16]);

    Store::print_blockchain(&blockchain);
}

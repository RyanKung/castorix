use std::process::Command;

use clap::Parser;
use clap::Subcommand;

#[derive(Parser)]
#[command(name = "start-node")]
#[command(about = "Start local blockchain nodes for testing")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start Optimism Anvil node (port 8545, chain ID 10)
    Op {
        /// Use fast mode with 1 second block time
        #[arg(short, long)]
        fast: bool,
    },
    /// Start Base Anvil node (port 8546, chain ID 8453)
    Base {
        /// Use fast mode with 1 second block time
        #[arg(short, long)]
        fast: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();

    match cli.command {
        Commands::Op { fast } => start_op_node(fast),
        Commands::Base { fast } => start_base_node(fast),
    }
}

fn start_op_node(fast: bool) {
    if fast {
        println!("🚀 Starting Optimism Anvil node (Fast Mode)...");
    } else {
        println!("🚀 Starting Optimism Anvil node...");
    }

    // Get the Optimism RPC URL from consts
    let fork_url = castorix::consts::get_config().eth_op_rpc_url().to_string();

    // Build Anvil arguments
    let mut args = vec![
        "--host",
        "127.0.0.1",
        "--port",
        "8545",
        "--accounts",
        "10",
        "--balance",
        "10000",
        "--gas-limit",
        "30000000",
        "--gas-price",
        "1000000000",
        "--chain-id",
        "10", // Optimism mainnet chain ID
        "--fork-url",
        &fork_url,
        "--fork-block-number",
        "latest", // Always start from latest block
        "--retries",
        "3",
        "--timeout",
        "10000",
    ];

    // Add fast mode options
    if fast {
        args.extend(["--block-time", "1"]);
    }

    args.push("--silent");

    // Start Anvil with Optimism fork configuration
    #[allow(clippy::zombie_processes)]
    let output = Command::new("anvil")
        .args(&args)
        .spawn()
        .expect("Failed to start Optimism Anvil - make sure it's installed");

    println!("✅ Optimism Anvil started with PID: {}", output.id());
    println!("📡 Optimism node running on http://127.0.0.1:8545");
    println!("🔗 Forking from: {}", fork_url);
    println!("⚡ Using latest block for fastest startup");
    if fast {
        println!("🏃 Fast mode: 1 second block time");
    }
}

fn start_base_node(fast: bool) {
    if fast {
        println!("🚀 Starting Base Anvil node (Fast Mode)...");
    } else {
        println!("🚀 Starting Base Anvil node...");
    }

    // Get the Base RPC URL from consts
    let fork_url = castorix::consts::get_config()
        .eth_base_rpc_url()
        .to_string();

    // Build Anvil arguments
    let mut args = vec![
        "--host",
        "127.0.0.1",
        "--port",
        "8546", // Different port for Base to avoid conflicts
        "--accounts",
        "10",
        "--balance",
        "10000",
        "--gas-limit",
        "30000000",
        "--gas-price",
        "1000000000",
        "--chain-id",
        "8453", // Base mainnet chain ID
        "--fork-url",
        &fork_url,
        "--fork-block-number",
        "latest", // Always start from latest block
        "--retries",
        "3",
        "--timeout",
        "10000",
    ];

    // Add fast mode options
    if fast {
        args.extend(["--block-time", "1"]);
    }

    args.push("--silent");

    // Start Anvil with Base fork configuration
    #[allow(clippy::zombie_processes)]
    let output = Command::new("anvil")
        .args(&args)
        .spawn()
        .expect("Failed to start Base Anvil - make sure it's installed");

    println!("✅ Base Anvil started with PID: {}", output.id());
    println!("📡 Base node running on http://127.0.0.1:8546");
    println!("🔗 Forking from: {}", fork_url);
    println!("⚡ Using latest block for fastest startup");
    if fast {
        println!("🏃 Fast mode: 1 second block time");
    }
}

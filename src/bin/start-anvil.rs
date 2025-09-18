use std::env;
use std::process::Command;

fn main() {
    println!("🚀 Starting Anvil node...");

    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();

    // Get the Optimism RPC URL from environment
    let fork_url =
        env::var("ETH_OP_RPC_URL").unwrap_or_else(|_| "https://rpc.ankr.com/optimism".to_string());

    // Start Anvil with fork configuration
    let output = Command::new("anvil")
        .args(&[
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
            "10",
            "--fork-url",
            &fork_url,
            "--silent",
        ])
        .spawn()
        .expect("Failed to start Anvil - make sure it's installed");

    println!("✅ Anvil started with PID: {}", output.id());
    println!("📡 Node running on http://127.0.0.1:8545");
    println!("🔗 Forking from: {}", fork_url);
}

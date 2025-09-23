use std::process::Command;

fn main() {
    println!("🚀 Starting Base Anvil node...");

    // Load environment variables from .env file if it exists
    dotenv::dotenv().ok();

    // Get the Base RPC URL from consts
    let fork_url = castorix::consts::get_config().eth_base_rpc_url().to_string();

    // Start Anvil with Base fork configuration
    #[allow(clippy::zombie_processes)]
    let output = Command::new("anvil")
        .args([
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
            "--silent",
        ])
        .spawn()
        .expect("Failed to start Base Anvil - make sure it's installed");

    println!("✅ Base Anvil started with PID: {}", output.id());
    println!("📡 Base node running on http://127.0.0.1:8546");
    println!("🔗 Forking from: {}", fork_url);
}

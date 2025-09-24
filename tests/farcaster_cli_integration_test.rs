use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

mod test_consts;
use test_consts::setup_local_test_env;
use test_consts::setup_placeholder_test_env;

/// Get the correct path to the castorix binary
fn get_castorix_binary() -> String {
    // Try different possible paths
    let possible_paths = vec![
        "./target/debug/castorix",
        "./target/release/castorix",
        "./target/aarch64-apple-darwin/debug/castorix",
        "./target/aarch64-apple-darwin/release/castorix",
        "./target/x86_64-unknown-linux-gnu/debug/castorix",
        "./target/x86_64-unknown-linux-gnu/release/castorix",
        "./target/x86_64-pc-windows-msvc/debug/castorix.exe",
        "./target/x86_64-pc-windows-msvc/release/castorix.exe",
    ];

    for path in possible_paths {
        if Path::new(path).exists() {
            return path.to_string();
        }
    }

    // Fallback to cargo run if no binary found
    "cargo run --bin castorix --".to_string()
}

/// Simplified CLI integration test using pre-built binary
///
/// This test covers the CLI workflow without rebuilding:
/// 1. Start local Anvil node
/// 2. Test FID price query
/// 3. Test storage price query
/// 4. Test FID listing
/// 5. Test storage usage
/// 6. Clean up
#[tokio::test]
async fn test_cli_integration_workflow() {

    println!("🚀 Starting CLI Integration Test");

    // Step 1: Start local Anvil node
    println!("📡 Starting local Anvil node...");
    let anvil_handle = start_local_anvil().await;

    // Give Anvil time to start if we started a new instance
    if anvil_handle.is_some() {
        thread::sleep(Duration::from_secs(3));
    }

    // Verify Anvil is running
    if !verify_anvil_running().await {
        panic!(
            "❌ Anvil failed to start - integration test cannot proceed without blockchain node"
        );
    }
    println!("✅ Anvil is running");

    // Set up local test environment
    setup_local_test_env();

    let test_fid = 460432; // Use a known test FID

    // Step 2: Test FID price query
    println!("\n💰 Testing FID Price Query...");
    test_command(&["fid", "price"], "FID price query", |output| {
        output.contains("ETH") || output.contains("Price")
    })
    .await;

    // Step 3: Test storage price query
    println!("\n🏠 Testing Storage Price Query...");
    test_command(
        &["storage", "price", &test_fid.to_string(), "--units", "5"],
        "Storage price query",
        |output| output.contains("ETH") || output.contains("Price"),
    )
    .await;

    // Step 4: Test FID listing
    println!("\n📋 Testing FID Listing...");
    test_command(&["fid", "list"], "FID listing", |output| {
        output.contains("FID") || output.contains("wallet")
    })
    .await;

    // Step 5: Test storage usage
    println!("\n📊 Testing Storage Usage...");
    test_command(
        &["storage", "usage", &test_fid.to_string()],
        "Storage usage query",
        |output| output.contains("FID") || output.contains("Storage"),
    )
    .await;

    // Step 6: Test help commands
    println!("\n📖 Testing Help Commands...");
    test_command(&["--help"], "Main help", |output| {
        output.contains("Usage:") || output.contains("Commands:")
    })
    .await;

    test_command(&["fid", "--help"], "FID help", |output| {
        output.contains("FID") || output.contains("Commands:")
    })
    .await;

    test_command(&["storage", "--help"], "Storage help", |output| {
        output.contains("Storage") || output.contains("Commands:")
    })
    .await;

    // Step 7: Test configuration validation
    println!("\n🔧 Testing Configuration Validation...");
    setup_placeholder_test_env();
    test_command(&["fid", "price"], "Configuration validation", |output| {
        output.contains("Warning") || output.contains("placeholder")
    })
    .await;

    // Reset configuration
    setup_local_test_env();

    // Clean up (only if we started a new instance)
    if let Some(handle) = anvil_handle {
        cleanup_anvil(Some(handle)).await;
    }

    println!("\n✅ CLI Integration Test Completed Successfully!");
}

/// Start local Anvil node
async fn start_local_anvil() -> Option<std::process::Child> {
    // Check if anvil is already running on port 8545
    if verify_anvil_running().await {
        println!("✅ Anvil is already running on port 8545");
        return None; // Return None to indicate we're using existing instance
    }

    // Start anvil with fork mode for Optimism
    println!("🔄 Starting Anvil with Optimism fork...");
    let anvil_output = Command::new("anvil")
        .args([
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
            "https://mainnet.optimism.io",
            "--retries",
            "3",
            "--timeout",
            "10000",
            "--block-time",
            "1",
            "--silent",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    match anvil_output {
        Ok(child) => {
            println!("✅ Anvil started with PID: {:?}", child.id());
            println!("🔗 Forking from Optimism mainnet");
            Some(child)
        }
        Err(e) => {
            println!("❌ Failed to start Anvil: {}", e);
            None
        }
    }
}

/// Verify Anvil is running by checking if it responds to RPC calls
async fn verify_anvil_running() -> bool {
    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_blockNumber",
        "params": [],
        "id": 1
    });

    match client
        .post("http://127.0.0.1:8545")
        .json(&payload)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                if let Ok(text) = response.text().await {
                    if text.contains("result") {
                        println!("✅ Anvil RPC is responding");
                        return true;
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Anvil RPC error: {}", e);
        }
    }

    false
}

/// Test a CLI command with expected output validation
async fn test_command<F>(args: &[&str], description: &str, validator: F)
where
    F: Fn(&str) -> bool,
{
    println!("   Testing {}...", description);

    // Use the correct binary path
    let binary_path = get_castorix_binary();
    let output = if binary_path.starts_with("cargo run") {
        // Use cargo run for the command
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "--bin", "castorix", "--"]);
        cmd.args(args);
        cmd.output()
    } else {
        // Use the binary directly
        Command::new(&binary_path).args(args).output()
    };

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() {
                if validator(&stdout) {
                    println!("   ✅ {} successful", description);
                    // Show a snippet of the output
                    if let Some(first_line) = stdout.lines().next() {
                        println!("   📝 Output: {}", first_line);
                    }
                } else {
                    panic!(
                        "❌ {} completed but output unexpected. Output: {}",
                        description,
                        if !stdout.is_empty() {
                            stdout.lines().take(2).collect::<Vec<_>>().join(" ")
                        } else {
                            "(empty)".to_string()
                        }
                    );
                }
            } else {
                panic!(
                    "❌ {} failed with status: {}. Error: {}",
                    description,
                    output.status,
                    if !stderr.is_empty() {
                        stderr.lines().take(2).collect::<Vec<_>>().join(" ")
                    } else {
                        "(no error output)".to_string()
                    }
                );
            }
        }
        Err(e) => {
            panic!("❌ {} command failed: {}", description, e);
        }
    }
}

/// Clean up Anvil process
async fn cleanup_anvil(anvil_handle: Option<std::process::Child>) {
    if let Some(mut handle) = anvil_handle {
        let _ = handle.kill();
        println!("🛑 Stopped local Anvil node");
    }
}

/// Test environment variable configuration
#[tokio::test]
async fn test_environment_configuration() {
    println!("🔧 Testing Environment Configuration...");

    // Test with placeholder values
    setup_placeholder_test_env();

    let output = Command::new(get_castorix_binary())
        .args(["fid", "price"])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Test passes if the command succeeds (even with placeholder config)
            // or if it shows configuration warnings
            if output.status.success()
                || stdout.contains("Configuration Warning")
                || stdout.contains("placeholder")
                || stderr.contains("Configuration Warning")
                || stderr.contains("placeholder")
            {
                println!("   ✅ Configuration validation working correctly");
            } else {
                panic!(
                    "❌ Configuration validation may not be working. Output: {}, Error: {}",
                    stdout, stderr
                );
            }
        }
        Err(e) => {
            panic!("❌ Configuration validation test failed: {}", e);
        }
    }

    // Reset configuration
    setup_local_test_env();
}

/// Test CLI argument parsing
#[tokio::test]
async fn test_cli_argument_parsing() {
    println!("🔧 Testing CLI Argument Parsing...");

    let test_cases = vec![
        (vec!["--help"], "Main help"),
        (vec!["fid", "--help"], "FID help"),
        (vec!["storage", "--help"], "Storage help"),
        (vec!["--version"], "Version"),
    ];

    for (args, description) in test_cases {
        println!("   Testing {}...", description);

        let output = Command::new(get_castorix_binary()).args(&args).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    println!("   ✅ {} working", description);
                    if let Some(first_line) = stdout.lines().next() {
                        println!("   📝 First line: {}", first_line);
                    }
                } else {
                    panic!("❌ {} failed with status: {}", description, output.status);
                }
            }
            Err(e) => {
                panic!("❌ {} test failed: {}", description, e);
            }
        }
    }
}

use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

mod test_consts;
use test_consts::setup_local_test_env;
use test_consts::setup_placeholder_test_env;

/// Complete Farcaster workflow integration test
///
/// This test covers the full workflow:
/// 1. Start local Anvil node
/// 2. Test FID registration
/// 3. Test storage rental
/// 4. Test signer registration
/// 5. Test signer deletion
/// 6. Clean up
#[tokio::test]
async fn test_complete_farcaster_workflow() {
    // Skip if no RPC tests should run
    if test_consts::should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }

    println!("🚀 Starting Complete Farcaster Workflow Test");

    // Clean up any existing test data
    let _ = std::fs::remove_dir_all("./test_data");

    // Step 1: Start local Anvil node
    println!("📡 Starting local Anvil node...");
    let anvil_handle = start_local_anvil().await;

    // Give Anvil time to start
    thread::sleep(Duration::from_secs(3));

    // Verify Anvil is running
    if !verify_anvil_running().await {
        panic!(
            "❌ Anvil failed to start - integration test cannot proceed without blockchain node"
        );
    }
    println!("✅ Anvil is running");

    // Set up local test environment
    setup_local_test_env();

    // We'll generate a temporary private key for this workflow
    // No need to set PRIVATE_KEY environment variable

    let test_wallet_name = "test-workflow-wallet";
    let _test_data_dir = "./test_data";
    let test_fid = 999999; // Use a high FID number to avoid conflicts

    // Step 2: Test FID registration
    println!("\n🆕 Testing FID Registration...");
    test_fid_registration(test_wallet_name, test_fid).await;

    // Step 3: Test storage rental
    println!("\n🏠 Testing Storage Rental...");
    test_storage_rental(test_fid).await;

    // Step 4: Test signer registration
    println!("\n🔐 Testing Signer Registration...");
    let signer_key = test_signer_registration(test_fid).await;

    // Step 5: Test signer deletion
    println!("\n🗑️ Testing Signer Deletion...");
    test_signer_deletion(test_fid, &signer_key).await;

    // Step 6: Test FID listing
    println!("\n📋 Testing FID Listing...");
    test_fid_listing().await;

    // Step 7: Test storage usage
    println!("\n📊 Testing Storage Usage...");
    test_storage_usage(test_fid).await;

    // Clean up
    cleanup_test_wallet(test_wallet_name).await;

    // Clean up test data directory
    let _ = std::fs::remove_dir_all("./test_data");
    println!("🗑️ Cleaned up test data directory");

    // Stop Anvil
    if let Some(mut handle) = anvil_handle {
        let _ = handle.kill();
        println!("🛑 Stopped local Anvil node");
    }

    println!("\n✅ Complete Farcaster Workflow Test Completed Successfully!");
}

/// Start local Anvil node
async fn start_local_anvil() -> Option<std::process::Child> {
    // First try to start using our start-node binary
    let output = Command::new("cargo")
        .args(["run", "--bin", "start-node", "op", "--fast"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match output {
        Ok(child) => {
            println!("✅ Anvil process started with PID: {:?}", child.id());
            Some(child)
        }
        Err(e) => {
            println!("❌ Failed to start Anvil via start-node: {}", e);

            // Fallback: try to start anvil directly
            println!("🔄 Trying direct anvil startup...");
            let direct_output = Command::new("anvil")
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
                    "--block-time",
                    "1",
                    "--silent",
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match direct_output {
                Ok(child) => {
                    println!("✅ Anvil started directly with PID: {:?}", child.id());
                    Some(child)
                }
                Err(e) => {
                    println!("❌ Failed to start Anvil directly: {}", e);
                    None
                }
            }
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

/// Test FID registration workflow
async fn test_fid_registration(_wallet_name: &str, _fid: u64) {
    println!("   🔑 Creating test wallet...");

    // Note: We don't set PRIVATE_KEY environment variable anymore
    // Instead, we'll use --wallet parameter or create wallets as needed
    println!("   ✅ Using wallet-based approach (no environment variables)");

    // Test FID price query
    println!("   💰 Testing FID price query...");
    let price_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "fid",
            "price",
        ])
        .output();

    match price_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ FID price query successful");
                println!(
                    "   📊 Price info: {}",
                    stdout.lines().find(|l| l.contains("ETH")).unwrap_or("N/A")
                );

                // Validate that the output contains expected price information
                assert!(
                    stdout.contains("Base Registration Price:") || stdout.contains("ETH"),
                    "FID price output should contain price information: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("FID price query failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to query FID price: {}", e);
        }
    }

    // Test FID registration (real registration) using environment variable
    println!("   🆕 Testing FID registration...");
    let register_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "fid",
            "register",
            "--yes",
        ])
        .output();

    match register_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ FID registration successful");
                println!(
                    "   📝 Registration result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("Total")
                            || l.contains("success")
                            || l.contains("registered"))
                        .unwrap_or("N/A")
                );

                // Validate that the output contains registration success information
                assert!(
                    stdout.contains("success")
                        || stdout.contains("registered")
                        || stdout.contains("transaction")
                        || stdout.contains("hash"),
                    "FID registration output should contain success information: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("FID registration failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to register FID: {}", e);
        }
    }
}

/// Test storage rental workflow
async fn test_storage_rental(fid: u64) {
    // Note: No environment variables needed for storage operations

    // Test storage price query
    println!("   💰 Testing storage price query...");
    let price_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "storage",
            "price",
            &fid.to_string(),
            "--units",
            "5",
        ])
        .output();

    match price_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ Storage price query successful");
                println!(
                    "   📊 Price info: {}",
                    stdout.lines().find(|l| l.contains("ETH")).unwrap_or("N/A")
                );

                // Validate that the output contains expected storage price information
                assert!(
                    stdout.contains("Rental Price:") || stdout.contains("ETH"),
                    "Storage price output should contain price information: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Storage price query failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to query storage price: {}", e);
        }
    }

    // Test storage rental (real rental)
    println!("   🏠 Testing storage rental...");
    let rent_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "storage",
            "rent",
            &fid.to_string(),
            "--units",
            "5",
            "--yes",
        ])
        .output();

    match rent_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ Storage rental successful");
                println!(
                    "   📝 Rental result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("Total")
                            || l.contains("success")
                            || l.contains("rented"))
                        .unwrap_or("N/A")
                );

                // Validate that the output contains rental success information
                assert!(
                    stdout.contains("success")
                        || stdout.contains("rented")
                        || stdout.contains("transaction")
                        || stdout.contains("hash"),
                    "Storage rental output should contain success information: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Storage rental failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to rent storage: {}", e);
        }
    }
}

/// Test signer registration and return the signer key
async fn test_signer_registration(fid: u64) -> String {
    println!("   🔐 Testing signer registration...");

    // Note: No environment variables needed for signer operations

    // List signers (we'll use this instead of generating)
    let signer_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "signers",
            "list",
        ])
        .output();

    let signer_key = match signer_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ Signer list successful");

                // Validate that the output contains signer information
                assert!(
                    stdout.contains("signer")
                        || stdout.contains("key")
                        || stdout.contains("No signers")
                        || stdout.contains("Ed25519"),
                    "Signer list output should contain signer information: {}",
                    stdout
                );

                // Extract the key from output if available
                let key_line = stdout
                    .lines()
                    .find(|l| l.contains("Private Key:") || l.contains("Key:"));
                match key_line {
                    Some(line) => {
                        let key = line.split(": ").nth(1).unwrap_or("").trim().to_string();
                        println!("   ✅ Signer key found: {}...", &key[..8]);
                        key
                    }
                    None => {
                        panic!("No signer keys found - test environment should have signer keys");
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Signer list failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to list signers: {}", e);
        }
    };

    // Test signer registration (real registration)
    println!("   📝 Testing signer registration...");
    let register_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "signers",
            "register",
            &fid.to_string(),
            "--yes",
        ])
        .output();

    match register_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ Signer registration successful");
                println!(
                    "   📝 Registration result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("FID:")
                            || l.contains("success")
                            || l.contains("registered"))
                        .unwrap_or("N/A")
                );

                // Validate that the output contains signer registration success information
                assert!(
                    stdout.contains("success")
                        || stdout.contains("registered")
                        || stdout.contains("transaction")
                        || stdout.contains("hash"),
                    "Signer registration output should contain success information: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Signer registration failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to register signer: {}", e);
        }
    }

    signer_key
}

/// Test signer deletion
async fn test_signer_deletion(fid: u64, signer_key: &str) {
    println!("   🗑️ Testing signer deletion...");

    // Note: No environment variables needed for signer deletion

    let delete_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "signers",
            "unregister",
            &fid.to_string(),
            "--key",
            signer_key,
            "--yes",
        ])
        .output();

    match delete_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ Signer deletion successful");
                println!(
                    "   📝 Deletion result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("FID:")
                            || l.contains("success")
                            || l.contains("unregistered"))
                        .unwrap_or("N/A")
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Signer deletion failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run signer deletion command: {}", e);
        }
    }
}

/// Test FID listing
async fn test_fid_listing() {
    println!("   📋 Testing FID listing...");

    let list_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "fid",
            "list",
        ])
        .output();

    match list_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ FID listing successful");

                // Validate that the output contains FID information
                assert!(
                    stdout.contains("FID")
                        || stdout.contains("wallet")
                        || stdout.contains("No wallet"),
                    "FID list output should contain FID information: {}",
                    stdout
                );

                if stdout.contains("No wallet found") {
                    panic!("No wallet found - test environment should have a wallet");
                } else {
                    println!(
                        "   📊 FID list: {}",
                        stdout.lines().find(|l| l.contains("FID:")).unwrap_or("N/A")
                    );
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("FID listing failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to list FIDs: {}", e);
        }
    }
}

/// Test storage usage query
async fn test_storage_usage(fid: u64) {
    println!("   📊 Testing storage usage query...");

    let usage_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "storage",
            "usage",
            &fid.to_string(),
        ])
        .output();

    match usage_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ Storage usage query successful");
                println!(
                    "   📊 Usage info: {}",
                    stdout.lines().find(|l| l.contains("FID:")).unwrap_or("N/A")
                );

                // Validate that the output contains storage usage information
                assert!(
                    stdout.contains("FID:")
                        || stdout.contains("Storage")
                        || stdout.contains("Usage"),
                    "Storage usage output should contain usage information: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Storage usage query failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to query storage usage: {}", e);
        }
    }
}

/// Clean up test wallet
async fn cleanup_test_wallet(wallet_name: &str) {
    println!("   🧹 Cleaning up test wallet...");

    let delete_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "key",
            "delete",
            wallet_name,
        ])
        .output();

    match delete_output {
        Ok(output) => {
            if output.status.success() {
                println!("   ✅ Test wallet cleaned up successfully");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Wallet cleanup failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Wallet cleanup failed: {}", e);
        }
    }
}

/// Test configuration validation
#[tokio::test]
async fn test_configuration_validation() {
    println!("🔧 Testing Configuration Validation...");

    // Test with placeholder values
    setup_placeholder_test_env();

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_data",
            "fid",
            "price",
        ])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Configuration Warning") {
                println!("   ✅ Configuration validation working correctly");
            } else {
                panic!("Configuration validation failed");
            }
        }
        Err(e) => {
            panic!("Configuration validation test failed: {}", e);
        }
    }
}

/// Test help commands
#[tokio::test]
async fn test_help_commands() {
    println!("📖 Testing Help Commands...");

    let help_commands = vec![
        ("--help", "Main help"),
        ("fid --help", "FID help"),
        ("storage --help", "Storage help"),
        ("signers --help", "Signers help"),
        ("key --help", "Key help"),
    ];

    for (args, description) in help_commands {
        println!("   Testing {}...", description);

        let mut cmd_args = vec!["run", "--bin", "castorix", "--", "--path", "./test_data"];
        cmd_args.extend(args.split(" "));
        let output = Command::new("cargo").args(&cmd_args).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains("Usage:") || stdout.contains("Commands:") {
                        println!("   ✅ {} help working", description);
                    } else {
                        panic!("{} help command failed", description);
                    }
                } else {
                    panic!(
                        "{} help command failed with non-zero exit code",
                        description
                    );
                }
            }
            Err(e) => {
                panic!("{} help test failed: {}", description, e);
            }
        }
    }
}

/// Test storage rental with separate payment wallet
#[tokio::test]
async fn test_storage_rental_with_payment_wallet() {
    // Skip if RPC tests are disabled
    if test_consts::should_skip_rpc_tests() {
        println!("⏭️  Skipping storage rental with payment wallet test (SKIP_RPC_TESTS set)");
        return;
    }

    println!("🏠 Starting Storage Rental with Payment Wallet Test");

    // Clean up any existing test data
    let _ = std::fs::remove_dir_all("./test_payment_wallet");

    // Step 1: Start local Anvil node
    println!("📡 Starting local Anvil node...");
    let anvil_handle = start_local_anvil().await;

    // Give Anvil time to start
    thread::sleep(Duration::from_secs(3));

    // Set up local test environment
    setup_local_test_env();

    let test_fid = 999999;
    let _custody_private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"; // Anvil account #0

    // Step 2: Test storage price query
    println!("💰 Testing storage price query...");
    let price_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_payment_wallet",
            "storage",
            "price",
            &test_fid.to_string(),
            "--units",
            "3",
        ])
        .output();

    match price_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("✅ Storage price query successful");
                println!(
                    "   📊 Price info: {}",
                    stdout.lines().find(|l| l.contains("ETH")).unwrap_or("N/A")
                );
                assert!(
                    stdout.contains("Rental Price:") || stdout.contains("ETH"),
                    "Storage price output should contain price information: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Storage price query failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to query storage price: {}", e);
        }
    }

    // Step 3: Test storage rental command structure (will fail due to missing wallets, but validates command parsing)
    println!("🏠 Testing storage rental command structure...");
    let rent_output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_payment_wallet",
            "storage",
            "rent",
            &test_fid.to_string(),
            "--units",
            "3",
            "--yes",
            "--wallet",
            "custody-wallet",
            "--payment-wallet",
            "payment-wallet",
        ])
        .output();

    match rent_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("✅ Storage rental with payment wallet successful");
                println!(
                    "   📝 Result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("success") || l.contains("rented"))
                        .unwrap_or("N/A")
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Storage rental failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run storage rental command: {}", e);
        }
    }

    // Clean up
    let _ = std::fs::remove_dir_all("./test_payment_wallet");
    println!("🗑️ Cleaned up test payment wallet directory");

    // Stop Anvil
    if let Some(mut handle) = anvil_handle {
        let _ = handle.kill();
        println!("🛑 Stopped local Anvil node");
    }

    println!("✅ Storage Rental with Payment Wallet Test Completed!");
}

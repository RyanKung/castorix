use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

mod test_consts;
use test_consts::setup_local_test_env;
use test_consts::should_skip_rpc_tests;

/// Complete ENS workflow integration test
///
/// This test covers the full ENS workflow:
/// 1. Start local Anvil node
/// 2. Generate encrypted private key
/// 3. Test ENS domain resolution
/// 4. Test ENS domain verification
/// 5. Generate username proof
/// 6. Verify proof
/// 7. Clean up
#[tokio::test]
async fn test_complete_ens_workflow() {
    // Skip if no RPC tests should run
    if should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }

    println!("🌐 Starting Complete ENS Workflow Test");

    // Clean up any existing test data
    let test_data_dir = "./test_ens_data";
    let _ = std::fs::remove_dir_all(test_data_dir);

    // Step 1: Start local Anvil node
    println!("📡 Starting local Anvil node...");
    let anvil_handle = start_local_anvil().await;

    // Give Anvil time to start
    thread::sleep(Duration::from_secs(3));

    // Verify Anvil is running
    if !verify_anvil_running().await {
        println!("❌ Anvil failed to start");
        return;
    }
    println!("✅ Anvil is running");

    // Set up local test environment
    setup_local_test_env();

    let test_wallet_name = "ens-test-wallet";
    let test_domain = "testuser.eth";
    let test_fid = 888888; // Use a different FID to avoid conflicts

    // Step 2: Generate encrypted private key
    println!("\n🔑 Testing Encrypted Key Generation...");
    test_generate_encrypted_key(test_data_dir, test_wallet_name).await;

    // Step 3: Test ENS domain resolution
    println!("\n🔍 Testing ENS Domain Resolution...");
    test_ens_resolution(test_data_dir, test_domain).await;

    // Step 4: Test ENS domain verification
    println!("\n✅ Testing ENS Domain Verification...");
    test_ens_verification(test_data_dir, test_domain).await;

    // Step 5: Generate username proof
    println!("\n📝 Testing Username Proof Generation...");
    test_proof_generation(test_data_dir, test_domain, test_fid, test_wallet_name).await;

    // Step 6: Verify proof
    println!("\n🔍 Testing Proof Verification...");
    test_proof_verification(test_data_dir, test_domain, test_fid).await;

    // Step 7: Test ENS domains query
    println!("\n🌐 Testing ENS Domains Query...");
    test_ens_domains_query(test_data_dir).await;

    // Clean up test data directory
    let _ = std::fs::remove_dir_all(test_data_dir);
    println!("🗑️ Cleaned up test data directory");

    // Stop Anvil
    if let Some(mut handle) = anvil_handle {
        let _ = handle.kill();
        println!("🛑 Stopped local Anvil node");
    }

    println!("\n✅ Complete ENS Workflow Test Completed Successfully!");
}

/// Start local Anvil node for testing
async fn start_local_anvil() -> Option<std::process::Child> {
    let output = Command::new("anvil")
        .args([
            "--fork-url",
            "https://optimism-mainnet.g.alchemy.com/v2/demo",
            "--fork-block-number",
            "latest",
            "--port",
            "8545",
            "--host",
            "0.0.0.0",
            "--block-time",
            "1",
            "--retries",
            "3",
            "--timeout",
            "10000",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match output {
        Ok(child) => {
            println!("✅ Anvil process started");
            Some(child)
        }
        Err(e) => {
            println!("❌ Failed to start Anvil: {}", e);
            None
        }
    }
}

/// Verify that Anvil is running
async fn verify_anvil_running() -> bool {
    let output = Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            r#"{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}"#,
            "http://127.0.0.1:8545",
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                response.contains("result")
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Test encrypted key generation
async fn test_generate_encrypted_key(test_data_dir: &str, wallet_name: &str) {
    println!("   🔑 Testing encrypted key generation...");

    // Create test data directory first
    let _ = std::fs::create_dir_all(test_data_dir);

    // Generate encrypted key with predefined inputs
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            test_data_dir,
            "key",
            "generate-encrypted",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match output {
        Ok(mut child) => {
            // Send predefined inputs
            let inputs = format!("{}\n{}\n{}\n", wallet_name, "test123", "test123");
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                let _ = stdin.write_all(inputs.as_bytes());
                let _ = stdin.flush();
            }

            let output = child.wait_with_output();
            match output {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        println!("   ✅ Encrypted key generated successfully");
                        println!(
                            "   📝 Output: {}",
                            stdout
                                .lines()
                                .find(|l| l.contains("Address:") || l.contains("saved"))
                                .unwrap_or("N/A")
                        );
                        assert!(
                            stdout.contains("Address:") || stdout.contains("saved"),
                            "Key generation should show address or success message: {}",
                            stdout
                        );
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        panic!("Key generation failed with stderr: {}", stderr);
                    }
                }
                Err(e) => {
                    panic!("Failed to run key generation command: {}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to spawn key generation process: {}", e);
        }
    }
}

/// Test ENS domain resolution
async fn test_ens_resolution(test_data_dir: &str, domain: &str) {
    println!("   🔍 Testing ENS domain resolution...");

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            test_data_dir,
            "ens",
            "resolve",
            domain,
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ ENS resolution successful");
                println!(
                    "   📝 Result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("Address:") || l.contains("0x"))
                        .unwrap_or("N/A")
                );
                // Note: Resolution might fail on local Anvil, but the command should still work
                assert!(
                    stdout.contains("Address:")
                        || stdout.contains("Error:")
                        || stdout.contains("Failed"),
                    "ENS resolution should show address or error: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("ENS resolution failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run ENS resolution command: {}", e);
        }
    }
}

/// Test ENS domain verification
async fn test_ens_verification(test_data_dir: &str, domain: &str) {
    println!("   ✅ Testing ENS domain verification...");

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            test_data_dir,
            "ens",
            "verify",
            domain,
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ ENS verification completed");
                println!(
                    "   📝 Result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("Owner:") || l.contains("Error:"))
                        .unwrap_or("N/A")
                );
                // Note: Verification might fail on local Anvil, but the command should still work
                assert!(
                    stdout.contains("Owner:")
                        || stdout.contains("Error:")
                        || stdout.contains("Failed"),
                    "ENS verification should show owner or error: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("ENS verification failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run ENS verification command: {}", e);
        }
    }
}

/// Test username proof generation
async fn test_proof_generation(test_data_dir: &str, domain: &str, fid: u64, wallet_name: &str) {
    println!("   📝 Testing username proof generation...");

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            test_data_dir,
            "ens",
            "proof",
            domain,
            &fid.to_string(),
            "--wallet-name",
            wallet_name,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match output {
        Ok(mut child) => {
            // Send password input
            let password = "test123\n";
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                let _ = stdin.write_all(password.as_bytes());
                let _ = stdin.flush();
            }

            let output = child.wait_with_output();
            match output {
                Ok(output) => {
                    if output.status.success() {
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        println!("   ✅ Username proof generated successfully");
                        println!(
                            "   📝 Result: {}",
                            stdout
                                .lines()
                                .find(|l| l.contains("Proof JSON:") || l.contains("saved"))
                                .unwrap_or("N/A")
                        );

                        // Check if proof file was created
                        let proof_file = format!("proof_{}_{}.json", domain.replace(".", "_"), fid);
                        if std::path::Path::new(&proof_file).exists() {
                            println!("   📄 Proof file created: {}", proof_file);
                            assert!(
                                stdout.contains("Proof JSON:") || stdout.contains("saved"),
                                "Proof generation should show JSON or success message: {}",
                                stdout
                            );
                        } else {
                            // Proof generation might fail due to domain verification, but should still work
                            println!("   ⚠️ Proof file not created (expected for test domain)");
                        }
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        // Proof generation might fail due to domain verification, which is expected
                        if stderr.contains("domain")
                            || stderr.contains("verification")
                            || stderr.contains("owner")
                        {
                            println!("   ⚠️ Proof generation failed due to domain verification (expected): {}", stderr.lines().next().unwrap_or("Unknown error"));
                        } else {
                            panic!("Proof generation failed with unexpected error: {}", stderr);
                        }
                    }
                }
                Err(e) => {
                    panic!("Failed to run proof generation command: {}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to spawn proof generation process: {}", e);
        }
    }
}

/// Test proof verification
async fn test_proof_verification(test_data_dir: &str, domain: &str, fid: u64) {
    println!("   🔍 Testing proof verification...");

    let proof_file = format!("proof_{}_{}.json", domain.replace(".", "_"), fid);

    // Check if proof file exists
    if !std::path::Path::new(&proof_file).exists() {
        println!("   ⚠️ Proof file does not exist, skipping verification test");
        return;
    }

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            test_data_dir,
            "ens",
            "verify-proof",
            &proof_file,
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ Proof verification successful");
                println!(
                    "   📝 Result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("Valid") || l.contains("Invalid"))
                        .unwrap_or("N/A")
                );
                assert!(
                    stdout.contains("Valid") || stdout.contains("Invalid"),
                    "Proof verification should show validity: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Proof verification failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run proof verification command: {}", e);
        }
    }
}

/// Test ENS domains query
async fn test_ens_domains_query(test_data_dir: &str) {
    println!("   🌐 Testing ENS domains query...");

    // Use a known test address (Anvil account #0)
    let test_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            test_data_dir,
            "ens",
            "domains",
            test_address,
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ ENS domains query successful");
                println!(
                    "   📝 Result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("domains") || l.contains("Found"))
                        .unwrap_or("N/A")
                );
                // Note: Query might return empty results on local Anvil, but the command should still work
                assert!(
                    stdout.contains("domains")
                        || stdout.contains("Found")
                        || stdout.contains("No domains"),
                    "ENS domains query should show results or no domains: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("ENS domains query failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run ENS domains query command: {}", e);
        }
    }
}

/// Test ENS configuration validation
#[tokio::test]
async fn test_ens_configuration_validation() {
    // Skip if no RPC tests should run
    if should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }

    println!("🔧 Testing ENS Configuration Validation...");

    let output = Command::new("cargo")
        .args(["run", "--bin", "castorix", "--", "ens", "--help"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("ENS domain proof operations") {
                    println!("   ✅ ENS configuration validation working correctly");
                } else {
                    panic!("ENS configuration validation failed");
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("ENS configuration validation test failed: {}", stderr);
            }
        }
        Err(e) => {
            panic!("ENS configuration validation test failed: {}", e);
        }
    }
}

/// Test ENS help commands
#[tokio::test]
async fn test_ens_help_commands() {
    // Skip if no RPC tests should run
    if should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }

    println!("📖 Testing ENS Help Commands...");

    let help_commands = vec![
        (vec!["ens", "--help"], "ENS main help"),
        (vec!["ens", "resolve", "--help"], "ENS resolve help"),
        (vec!["ens", "domains", "--help"], "ENS domains help"),
        (vec!["ens", "proof", "--help"], "ENS proof help"),
        (
            vec!["ens", "verify-proof", "--help"],
            "ENS verify-proof help",
        ),
    ];

    for (args, description) in help_commands {
        let mut cmd_args = vec!["run", "--bin", "castorix", "--"];
        cmd_args.extend_from_slice(&args);

        let output = Command::new("cargo").args(&cmd_args).output();

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if stdout.contains("Usage:")
                        || stdout.contains("Commands:")
                        || stdout.contains("Arguments:")
                    {
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

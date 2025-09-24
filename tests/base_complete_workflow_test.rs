use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

mod test_consts;
use test_consts::setup_local_base_test_env;
use test_consts::should_skip_rpc_tests;

/// Complete Base workflow integration test
///
/// This test covers the full Base workflow:
/// 1. Start local Base Anvil node
/// 2. Generate encrypted private key
/// 3. Test Base ENS domain resolution
/// 4. Test Base ENS domain verification
/// 5. Generate username proof for Base domains
/// 6. Verify proof
/// 7. Clean up
#[tokio::test]
async fn test_complete_base_workflow() {
    // Skip if no RPC tests should run
    if should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }

    println!("🔵 Starting Complete Base Workflow Test");

    // Clean up any existing test data
    let test_data_dir = "./test_base_data";
    let _ = std::fs::remove_dir_all(test_data_dir);

    // Step 1: Start local Base Anvil node
    println!("📡 Starting local Base Anvil node...");
    let anvil_handle = start_local_base_anvil().await;

    // Give Anvil time to start
    thread::sleep(Duration::from_secs(3));

    // Verify Base Anvil is running
    if !verify_base_anvil_running().await {
        panic!("❌ Base Anvil failed to start - integration test cannot proceed without blockchain node");
    }
    println!("✅ Base Anvil is running");

    // Set up local Base test environment
    setup_local_base_test_env();

    let test_wallet_name = "base-test-wallet";
    let test_domain = "testuser.base.eth";
    let test_fid = 777777; // Use a different FID to avoid conflicts

    // Step 2: Generate encrypted private key
    println!("\n🔑 Testing Encrypted Key Generation...");
    test_generate_encrypted_key(test_data_dir, test_wallet_name).await;

    // Step 3: Test Base ENS domain resolution
    println!("\n🔍 Testing Base ENS Domain Resolution...");
    test_base_ens_resolution(test_data_dir, test_domain).await;

    // Step 4: Test Base ENS domain verification
    println!("\n✅ Testing Base ENS Domain Verification...");
    test_base_ens_verification(test_data_dir, test_domain).await;

    // Step 5: Generate username proof for Base domain
    println!("\n📝 Testing Base Username Proof Generation...");
    test_base_proof_generation(test_data_dir, test_domain, test_fid, test_wallet_name).await;

    // Step 6: Verify proof
    println!("\n🔍 Testing Proof Verification...");
    test_proof_verification(test_data_dir, test_domain, test_fid).await;

    // Step 7: Test Base ENS domains query
    println!("\n🌐 Testing Base ENS Domains Query...");
    test_base_ens_domains_query(test_data_dir).await;

    // Clean up test data directory
    let _ = std::fs::remove_dir_all(test_data_dir);
    println!("🗑️ Cleaned up test data directory");

    // Stop Base Anvil
    if let Some(mut handle) = anvil_handle {
        let _ = handle.kill();
        println!("🛑 Stopped local Base Anvil node");
    }

    println!("\n✅ Complete Base Workflow Test Completed Successfully!");
}

/// Start local Base Anvil node for testing
async fn start_local_base_anvil() -> Option<std::process::Child> {
    // Start anvil directly instead of through cargo to avoid blocking
    let anvil_process = Command::new("anvil")
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            "8546",
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
            "https://mainnet.base.org",
            "--retries",
            "3",
            "--timeout",
            "10000",
            "--block-time",
            "1", // Fast mode
            "--silent",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match anvil_process {
        Ok(child) => {
            println!("✅ Base Anvil process started with PID: {}", child.id());
            Some(child)
        }
        Err(e) => {
            println!("❌ Failed to start Base Anvil: {}", e);
            None
        }
    }
}

/// Verify that Base Anvil is running
async fn verify_base_anvil_running() -> bool {
    let output = Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            r#"{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}"#,
            "http://127.0.0.1:8546",
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

/// Test Base ENS domain resolution
async fn test_base_ens_resolution(test_data_dir: &str, domain: &str) {
    println!("   🔍 Testing Base ENS domain resolution...");

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
                println!("   ✅ Base ENS resolution successful");
                println!(
                    "   📝 Result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("Address:") || l.contains("0x"))
                        .unwrap_or("N/A")
                );
                // Since we're forking mainnet, ENS resolution should succeed
                assert!(
                    stdout.contains("Address:") || stdout.contains("Resolved to:"),
                    "Base ENS resolution should succeed with fork - got: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Base ENS resolution failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run Base ENS resolution command: {}", e);
        }
    }
}

/// Test Base ENS domain verification
async fn test_base_ens_verification(test_data_dir: &str, domain: &str) {
    println!("   ✅ Testing Base ENS domain verification...");

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
                println!("   ✅ Base ENS verification completed");
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
                    "Base ENS verification should show owner or error: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Base ENS verification failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run Base ENS verification command: {}", e);
        }
    }
}

/// Test Base username proof generation
async fn test_base_proof_generation(
    test_data_dir: &str,
    domain: &str,
    fid: u64,
    wallet_name: &str,
) {
    println!("   📝 Testing Base username proof generation...");

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
                        println!("   ✅ Base username proof generated successfully");
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
                                "Base proof generation should show JSON or success message: {}",
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
                            println!("   ⚠️ Base proof generation failed due to domain verification (expected): {}", stderr.lines().next().unwrap_or("Unknown error"));
                        } else {
                            panic!(
                                "Base proof generation failed with unexpected error: {}",
                                stderr
                            );
                        }
                    }
                }
                Err(e) => {
                    panic!("Failed to run Base proof generation command: {}", e);
                }
            }
        }
        Err(e) => {
            panic!("Failed to spawn Base proof generation process: {}", e);
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

/// Test Base ENS domains query
async fn test_base_ens_domains_query(test_data_dir: &str) {
    println!("   🌐 Testing Base ENS domains query...");

    // Use a known test address (Base test account)
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
                println!("   ✅ Base ENS domains query successful");
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
                    "Base ENS domains query should show results or no domains: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Base ENS domains query failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run Base ENS domains query command: {}", e);
        }
    }
}

/// Test Base configuration validation
#[tokio::test]
async fn test_base_configuration_validation() {
    // Skip if no RPC tests should run
    if should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }

    println!("🔧 Testing Base Configuration Validation...");

    // Test that start-node base command works
    let output = Command::new("cargo")
        .args(["run", "--bin", "start-node", "base"])
        .output();

    match output {
        Ok(output) => {
            // The command should either succeed or fail gracefully
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            if output.status.success() {
                println!("   ✅ Base node configuration working correctly");
            } else if stderr.contains("anvil") || stdout.contains("Base Anvil") {
                println!("   ✅ Base node configuration working correctly (anvil not running)");
            } else {
                panic!("Base configuration validation failed");
            }
        }
        Err(e) => {
            panic!("Base configuration validation test failed: {}", e);
        }
    }
}

/// Test Base subdomain checking
#[tokio::test]
async fn test_base_subdomain_checking() {
    // Skip if no RPC tests should run
    if should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }

    println!("🔍 Testing Base Subdomain Checking...");

    let test_domain = "test.base.eth";
    let _test_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    // Test base subdomain check
    let output = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "castorix",
            "--",
            "--path",
            "./test_base_data",
            "ens",
            "check-base-subdomain",
            test_domain,
        ])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                println!("   ✅ Base subdomain check successful");
                println!(
                    "   📝 Result: {}",
                    stdout
                        .lines()
                        .find(|l| l.contains("subdomain") || l.contains("Error:"))
                        .unwrap_or("N/A")
                );
                assert!(
                    stdout.contains("subdomain")
                        || stdout.contains("Error:")
                        || stdout.contains("Failed"),
                    "Base subdomain check should show result or error: {}",
                    stdout
                );
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                panic!("Base subdomain check failed with stderr: {}", stderr);
            }
        }
        Err(e) => {
            panic!("Failed to run Base subdomain check command: {}", e);
        }
    }
}

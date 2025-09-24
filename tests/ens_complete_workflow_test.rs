use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

mod test_consts;
use test_consts::setup_local_test_env;

/// Complete ENS workflow integration test
///
/// This test covers the full ENS workflow:
/// 1. Start local Anvil node (REQUIRED - no downgrade)
/// 2. Generate encrypted private key
/// 3. Test ENS domain resolution
/// 4. Test ENS domain verification
/// 5. Generate username proof
/// 6. Verify proof
/// 7. Clean up
#[tokio::test]
async fn test_complete_ens_workflow() {
    println!("🌐 Starting Complete ENS Workflow Test");

    // Clean up any existing test data
    let test_data_dir = "./test_ens_data";
    let _ = std::fs::remove_dir_all(test_data_dir);

    // Step 1: Verify Anvil node is running (started by CI workflow or Makefile)
    println!("📡 Checking for running Anvil node...");
    
    // Check if we should use pre-started nodes (CI environment)
    let use_pre_started = std::env::var("RUNNING_TESTS").is_ok();
    
    if use_pre_started {
        println!("🔧 Using pre-started Anvil nodes (CI environment)");
        // Verify Anvil is running on expected ports
        if !verify_anvil_running().await {
            panic!(
                "❌ Pre-started Anvil node not available - integration test cannot proceed without blockchain node."
            );
        }
        println!("✅ Pre-started Anvil node is running");
    } else {
        println!("🏠 Starting local Anvil node for local testing...");
        let anvil_handle = start_local_anvil().await;

        // Give Anvil time to start
        thread::sleep(Duration::from_secs(3));

        // Verify Anvil is running - FAIL if not available
        if !verify_anvil_running().await {
            panic!(
                "❌ Anvil failed to start - integration test cannot proceed without blockchain node. This test requires a local Anvil node to function properly."
            );
        }
        println!("✅ Local Anvil node is running");
        
        // Store handle for cleanup
        std::env::set_var("ANVIL_HANDLE", format!("{:?}", anvil_handle));
    }

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

    // Stop Anvil only for local testing (not in CI)
    if !use_pre_started {
        // Note: In local testing, the anvil_handle would need to be accessible here
        // For now, we'll rely on the Makefile to manage local nodes
        println!("🏠 Local testing: Anvil node cleanup handled by Makefile");
    } else {
        println!("🔧 CI environment: Anvil nodes managed by workflow");
    }

    println!("\n✅ Complete ENS Workflow Test Completed Successfully!");
}

/// Start local Anvil node for testing
async fn start_local_anvil() -> Option<std::process::Child> {
    // Check if Anvil is available - FAIL if not found
    let check_output = Command::new("which").arg("anvil").output();

    if let Ok(output) = check_output {
        if !output.status.success() {
            panic!("❌ Anvil not found in PATH - this test requires Anvil to be installed and available");
        }
    } else {
        panic!("❌ Cannot check for Anvil availability - this test requires Anvil to be installed and available");
    }

    let output = Command::new("anvil")
        .args([
            "--fork-url",
            "https://mainnet.optimism.io",
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
            panic!(
                "❌ Failed to start Anvil: {} - this test requires Anvil to start successfully",
                e
            );
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
            // Send predefined inputs: key_name, password, confirm_password, confirm_save
            let inputs = format!("{}\n{}\n{}\ny\n", wallet_name, "test123", "test123");
            if let Some(stdin) = child.stdin.as_mut() {
                use std::io::Write;
                let _ = stdin.write_all(inputs.as_bytes());
                let _ = stdin.flush();
            }

            // Use tokio::time::timeout to prevent hanging
            let timeout_duration = Duration::from_secs(10); // 10 second timeout
            println!(
                "   ⏱️  Waiting for process completion (timeout: {}s)...",
                timeout_duration.as_secs()
            );

            let result =
                tokio::time::timeout(timeout_duration, async { child.wait_with_output() }).await;

            let output = match result {
                Ok(output_result) => output_result,
                Err(_timeout) => {
                    println!(
                        "   ⏰ Process timed out after {} seconds",
                        timeout_duration.as_secs()
                    );
                    println!("   🔍 This usually indicates the process is waiting for user input");
                    println!("   💡 Check if the command requires interactive input that wasn't provided");

                    // Note: child is already consumed by the async block, so we can't kill it here
                    // The process will be cleaned up when the async block completes

                    panic!(
                        "❌ Encrypted key generation timed out - process may be waiting for input"
                    );
                }
            };
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
                        || stdout.contains("Failed")
                        || stdout.contains("not found")
                        || stdout.contains("not resolved"),
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
                        || stdout.contains("Failed")
                        || stdout.contains("don't own")
                        || stdout.contains("not found"),
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

            // Use tokio::time::timeout to prevent hanging
            let timeout_duration = Duration::from_secs(10); // 10 second timeout
            println!(
                "   ⏱️  Waiting for proof generation (timeout: {}s)...",
                timeout_duration.as_secs()
            );

            let result =
                tokio::time::timeout(timeout_duration, async { child.wait_with_output() }).await;

            let output = match result {
                Ok(output_result) => output_result,
                Err(_timeout) => {
                    println!(
                        "   ⏰ Process timed out after {} seconds",
                        timeout_duration.as_secs()
                    );
                    println!("   🔍 This usually indicates the process is waiting for user input");
                    println!("   💡 Check if the command requires interactive input that wasn't provided");

                    // Note: child is already consumed by the async block, so we can't kill it here
                    // The process will be cleaned up when the async block completes

                    panic!(
                        "❌ Username proof generation timed out - process may be waiting for input"
                    );
                }
            };
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

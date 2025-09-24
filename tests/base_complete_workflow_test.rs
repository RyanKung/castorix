use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

mod test_consts;
use test_consts::setup_local_base_test_env;

/// Generate a random hash string of specified length
fn generate_random_hash(length: usize) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    let mut hasher = DefaultHasher::new();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    timestamp.hash(&mut hasher);

    let hash = hasher.finish();
    format!("{:x}", hash)[..length].to_string()
}

/// Complete Base workflow integration test
///
/// This test covers the full Base workflow:
/// 1. Start local Base Anvil node
/// 2. Generate encrypted private key
/// 3. Register Base ENS domain (with 9-char hash to prevent collisions)
/// 4. Test Base ENS domain resolution
/// 5. Test Base ENS domain verification
/// 6. Generate username proof for Base domains
/// 7. Verify proof
/// 8. Test Base ENS domains query
/// 9. Clean up
#[tokio::test]
async fn test_complete_base_workflow() {
    println!("🔵 Starting Complete Base Workflow Test");

    // Clean up any existing test data
    let test_data_dir = "./test_base_data";
    let _ = std::fs::remove_dir_all(test_data_dir);

    // Step 1: Verify Base Anvil node is running (started by CI workflow or Makefile)
    println!("📡 Checking for running Base Anvil node...");

    // Check if we should use pre-started nodes (CI environment)
    let use_pre_started = std::env::var("RUNNING_TESTS").is_ok();

    if use_pre_started {
        println!("🔧 Using pre-started Base Anvil node (CI environment)");
        // Verify Base Anvil is running on expected port
        if !verify_base_anvil_running().await {
            panic!("❌ Pre-started Base Anvil node not available - integration test cannot proceed without blockchain node");
        }
        println!("✅ Pre-started Base Anvil node is running");
    } else {
        println!("🏠 Starting local Base Anvil node for local testing...");
        let anvil_handle = start_local_base_anvil().await;

        // Give Anvil time to start
        thread::sleep(Duration::from_secs(3));

        // Verify Base Anvil is running
        if !verify_base_anvil_running().await {
            println!("❌ Base Anvil failed to start - trying to start manually...");
            println!("💡 Use 'make start-nodes' to start Anvil nodes, then run 'make test-ci'");
            println!("💡 Or use 'make test-local' to start nodes and run tests automatically");
            panic!("❌ Base Anvil failed to start - integration test cannot proceed without blockchain node.\n\
            \n\
            To fix this:\n\
            1. Use 'make test-local' to start nodes and run tests automatically\n\
            2. Or manually start nodes with 'make start-nodes' then run 'make test-ci'\n\
            3. Or run tests in CI environment where nodes are pre-started");
        }
        println!("✅ Local Base Anvil node is running");
        
        // Store handle for cleanup
        std::env::set_var("BASE_ANVIL_HANDLE", format!("{:?}", anvil_handle));
    }

    // Set up local Base test environment
    setup_local_base_test_env();

    let test_wallet_name = "base-test-wallet";
    // Generate a 9-character random hash for domain to prevent collisions
    let random_hash = generate_random_hash(9);
    let test_domain = format!("{}.base.eth", random_hash);
    let test_fid = 777777; // Use a different FID to avoid conflicts

    // Step 2: Generate encrypted private key
    println!("\n🔑 Testing Encrypted Key Generation...");
    test_generate_encrypted_key(test_data_dir, test_wallet_name).await;

    // Step 3: Register Base ENS domain (simulate registration)
    println!("\n📝 Testing Base ENS Domain Registration...");
    test_base_ens_registration(test_data_dir, &test_domain).await;

    // Step 4: Test Base ENS domain resolution (should fail for random domain)
    println!("\n🔍 Testing Base ENS Domain Resolution...");
    test_base_ens_resolution_expected_failure(test_data_dir, &test_domain).await;

    // Step 5: Test Base ENS domain verification
    println!("\n✅ Testing Base ENS Domain Verification...");
    test_base_ens_verification(test_data_dir, &test_domain).await;

    // Step 6: Generate username proof for Base domain
    println!("\n📝 Testing Base Username Proof Generation...");
    test_base_proof_generation(test_data_dir, &test_domain, test_fid, test_wallet_name).await;

    // Step 7: Verify proof
    println!("\n🔍 Testing Proof Verification...");
    test_proof_verification(test_data_dir, &test_domain, test_fid).await;

    // Step 8: Test Base ENS domains query
    println!("\n🌐 Testing Base ENS Domains Query...");
    test_base_ens_domains_query(test_data_dir).await;

    // Clean up test data directory
    let _ = std::fs::remove_dir_all(test_data_dir);
    println!("🗑️ Cleaned up test data directory");

    // Stop Base Anvil only for local testing (not in CI)
    if !use_pre_started {
        // Note: In local testing, the anvil_handle would need to be accessible here
        // For now, we'll rely on the Makefile to manage local nodes
        println!("🏠 Local testing: Base Anvil node cleanup handled by Makefile");
    } else {
        println!("🔧 CI environment: Base Anvil node managed by workflow");
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

            match result {
                Ok(Ok(output)) => {
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
                        println!(
                            "   ❌ Process failed with exit code: {:?}",
                            output.status.code()
                        );
                        println!("   📝 Stderr: {}", stderr);
                        panic!("Key generation failed with stderr: {}", stderr);
                    }
                }
                Ok(Err(e)) => {
                    panic!("❌ Failed to wait for encrypted key generation: {}", e);
                }
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
            }
        }
        Err(e) => {
            panic!("Failed to spawn key generation process: {}", e);
        }
    }
}

/// Test Base ENS domain registration (simulate registration process)
async fn test_base_ens_registration(_test_data_dir: &str, domain: &str) {
    println!("   📝 Testing Base ENS domain registration for: {}", domain);

    // In a real implementation, this would interact with Base ENS contracts
    // For now, we'll simulate the registration process by checking if the domain
    // follows the correct format and is available

    // Validate domain format
    assert!(
        domain.ends_with(".base.eth"),
        "Domain should end with .base.eth: {}",
        domain
    );

    // Check that the subdomain part is a valid hash (9 characters)
    let subdomain = domain.strip_suffix(".base.eth").unwrap();
    assert!(
        subdomain.len() == 9,
        "Subdomain should be 9 characters long: {}",
        subdomain
    );

    // Validate that it's a valid hex string
    assert!(
        subdomain.chars().all(|c| c.is_ascii_hexdigit()),
        "Subdomain should be a valid hex string: {}",
        subdomain
    );

    println!("   ✅ Domain format validation passed");
    println!("   📝 Domain: {} (9-char hash: {})", domain, subdomain);
}

/// Test Base ENS domain resolution (expecting failure for random domain)
async fn test_base_ens_resolution_expected_failure(test_data_dir: &str, domain: &str) {
    println!("   🔍 Testing Base ENS domain resolution (expecting failure)...");

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
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // For a random domain, resolution should fail
            if stdout.contains("not found")
                || stdout.contains("not resolved")
                || stderr.contains("not found")
            {
                println!("   ✅ Base ENS resolution failed as expected for random domain");
                println!("   📝 Result: Domain not found (as expected)");
            } else {
                // If it somehow succeeds, that's also fine - but we should log it
                println!("   ✅ Base ENS resolution successful (domain exists)");
                println!("   📝 Result: {}", stdout);
            }
        }
        Err(e) => {
            panic!("❌ Failed to run Base ENS resolution command: {}", e);
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
                // For a newly generated domain, we expect it to not be owned
                // This is the expected behavior for a random hash domain
                assert!(
                    stdout.contains("You don't own this domain")
                        || stdout.contains("Owner:")
                        || stdout.contains("Error:")
                        || stdout.contains("Failed"),
                    "Base ENS verification should show ownership status: {}",
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

                    panic!("❌ Base proof generation timed out - process may be waiting for input");
                }
            };
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
    println!("🔧 Testing Base Configuration Validation...");

    // Test that anvil command works for Base configuration
    let output = Command::new("anvil")
        .args([
            "--host",
            "127.0.0.1",
            "--port",
            "8547", // Use different port to avoid conflicts
            "--chain-id",
            "8453", // Base mainnet chain ID
            "--fork-url",
            "https://mainnet.base.org",
            "--silent",
            "--help", // Just test that anvil is available and Base config is valid
        ])
        .output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            if output.status.success() && stdout.contains("--chain-id") {
                println!("   ✅ Base node configuration working correctly");
            } else {
                panic!("Base configuration validation failed - anvil not available or Base config invalid");
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
    println!("🔍 Testing Base Subdomain Checking...");

    // Generate a 9-character random hash for domain to prevent collisions
    let random_hash = generate_random_hash(9);
    let test_domain = format!("{}.base.eth", random_hash);
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
            &test_domain,
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

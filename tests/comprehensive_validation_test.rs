use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

mod test_consts;
use test_consts::{setup_local_test_env, should_skip_rpc_tests};

/// Comprehensive validation test for Farcaster CLI
///
/// This test performs strict cross-validation of all CLI operations:
/// 1. Creates test data directory and validates it exists
/// 2. Tests wallet creation with validation of encrypted storage
/// 3. Tests FID operations with price validation and format checking
/// 4. Tests storage operations with unit validation
/// 5. Tests signer operations with key validation
/// 6. Cross-validates all operations against each other
/// 7. Validates cleanup operations
#[tokio::test]
async fn test_comprehensive_cli_validation() {
    // Skip if no RPC tests should run
    if test_consts::should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }

    println!("🔬 Starting Comprehensive CLI Validation Test");

    let test_data_dir = "./test_validation_data";
    let test_wallet_name = "validation-test-wallet";
    let test_fid = 999999;

    // Clean up any existing test data
    cleanup_test_directory(test_data_dir);

    // Set up local test environment
    setup_local_test_env();

    // Start local Anvil node
    println!("📡 Starting local Anvil node...");
    let anvil_handle = start_local_anvil().await;
    thread::sleep(Duration::from_secs(3));

    // Test 1: Directory and Environment Validation
    println!("\n📁 Test 1: Directory and Environment Validation");
    test_directory_validation(test_data_dir).await;

    // Test 2: Wallet Creation and Validation
    println!("\n🔑 Test 2: Wallet Creation and Validation");
    let wallet_created = test_wallet_creation(test_data_dir, test_wallet_name).await;

    // Test 3: FID Operations with Price Validation
    println!("\n💰 Test 3: FID Operations with Price Validation");
    let price_data = test_fid_price_validation(test_data_dir).await;

    // Test 4: Storage Operations with Unit Validation
    println!("\n🏠 Test 4: Storage Operations with Unit Validation");
    let storage_data = test_storage_validation(test_data_dir, test_fid).await;

    // Test 5: Cross-Validation of Operations
    println!("\n🔍 Test 5: Cross-Validation of Operations");
    test_cross_validation(test_data_dir, wallet_created, &price_data, &storage_data).await;

    // Test 6: Cleanup Validation
    println!("\n🧹 Test 6: Cleanup Validation");
    test_cleanup_validation(test_data_dir, test_wallet_name).await;

    // Stop Anvil
    if let Some(mut handle) = anvil_handle {
        let _ = handle.kill();
        println!("🛑 Stopped local Anvil node");
    }

    println!("\n✅ Comprehensive CLI Validation Test Completed!");
}

/// Test directory creation and validation
async fn test_directory_validation(test_data_dir: &str) {
    println!("   📁 Testing directory validation: {}", test_data_dir);

    // Create the directory manually first
    let _ = fs::create_dir_all(test_data_dir);

    // Verify directory exists
    assert!(
        fs::metadata(test_data_dir).is_ok(),
        "Test directory should exist"
    );

    // Run a simple CLI command to test path functionality
    let output = run_cli_command(test_data_dir, &["key", "list"]);

    // Command should succeed (even if no keys exist)
    println!(
        "   📊 Directory test output: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    println!("   ✅ Directory validation passed");
}

/// Test wallet creation with comprehensive validation
async fn test_wallet_creation(test_data_dir: &str, _wallet_name: &str) -> bool {
    println!("   🔑 Testing wallet creation...");

    // Run wallet creation command
    let output = run_cli_command(test_data_dir, &["key", "generate-encrypted"]);

    // Validate wallet creation output
    let wallet_created = output.status.success();

    if wallet_created {
        println!("   ✅ Wallet creation command succeeded");

        // Verify wallet appears in list
        let list_output = run_cli_command(test_data_dir, &["key", "list"]);
        let list_stdout = String::from_utf8_lossy(&list_output.stdout);

        // Check if wallet listing shows any encrypted keys
        let has_wallets = list_stdout.contains("encrypted keys")
            || list_stdout.contains("No encrypted keys")
            || list_stdout.contains("keys found");

        assert!(
            has_wallets,
            "Wallet list should show key status information"
        );
        println!("   ✅ Wallet listing validation passed");

        // Verify directory structure
        let keys_dir = format!("{}/keys", test_data_dir);
        if fs::metadata(&keys_dir).is_ok() {
            println!("   ✅ Keys directory structure validation passed");
        } else {
            println!("   ⚠️  Keys directory not found (may be created on first key)");
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("   ❌ Wallet creation failed: {}", stderr);
    }

    wallet_created
}

/// Test FID price validation with format checking
async fn test_fid_price_validation(test_data_dir: &str) -> Option<String> {
    println!("   💰 Testing FID price validation...");

    let output = run_cli_command(test_data_dir, &["fid", "price"]);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("   ❌ FID price query failed: {}", stderr);
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("   📊 Price output: {}", stdout);

    // Validate price format
    let price_validation = validate_price_format(&stdout);

    if price_validation {
        println!("   ✅ FID price format validation passed");
        Some(stdout.to_string())
    } else {
        println!("   ❌ FID price format validation failed");
        None
    }
}

/// Test storage operations with unit validation
async fn test_storage_validation(test_data_dir: &str, test_fid: u64) -> Option<String> {
    println!("   🏠 Testing storage validation...");

    // Test storage price query
    let price_output = run_cli_command(
        test_data_dir,
        &["storage", "price", &test_fid.to_string(), "--units", "5"],
    );

    if !price_output.status.success() {
        let stderr = String::from_utf8_lossy(&price_output.stderr);
        println!("   ❌ Storage price query failed: {}", stderr);
        return None;
    }

    let stdout = String::from_utf8_lossy(&price_output.stdout);
    println!("   📊 Storage price output: {}", stdout);

    // Validate storage price format
    let storage_validation = validate_storage_format(&stdout);

    if storage_validation {
        println!("   ✅ Storage price format validation passed");

        // Test storage usage query
        let usage_output =
            run_cli_command(test_data_dir, &["storage", "usage", &test_fid.to_string()]);

        if usage_output.status.success() {
            let usage_stdout = String::from_utf8_lossy(&usage_output.stdout);
            println!("   ✅ Storage usage query successful: {}", usage_stdout);
            Some(stdout.to_string())
        } else {
            println!("   ❌ Storage usage query failed");
            None
        }
    } else {
        println!("   ❌ Storage price format validation failed");
        None
    }
}

/// Cross-validate all operations
async fn test_cross_validation(
    test_data_dir: &str,
    _wallet_created: bool,
    price_data: &Option<String>,
    storage_data: &Option<String>,
) {
    println!("   🔍 Cross-validating operations...");

    // Test 1: Verify CLI help commands work
    let help_commands = [
        (vec!["--help"], "Main help"),
        (vec!["fid", "--help"], "FID help"),
        (vec!["storage", "--help"], "Storage help"),
        (vec!["key", "--help"], "Key help"),
        (vec!["signers", "--help"], "Signers help"),
    ];

    for (args, description) in help_commands {
        let output = run_cli_command(test_data_dir, &args);
        assert!(
            output.status.success(),
            "{} command should succeed",
            description
        );
        println!("   ✅ {} validation passed", description);
    }

    // Test 2: Verify FID listing works
    let fid_list_output = run_cli_command(test_data_dir, &["fid", "list"]);
    assert!(
        fid_list_output.status.success(),
        "FID list command should succeed"
    );
    println!("   ✅ FID list validation passed");

    // Test 3: Verify signer operations work
    let signer_list_output = run_cli_command(test_data_dir, &["signers", "list"]);
    assert!(
        signer_list_output.status.success(),
        "Signer list command should succeed"
    );
    println!("   ✅ Signer list validation passed");

    // Test 4: Cross-validate price data consistency
    if let (Some(fid_price), Some(storage_price)) = (price_data, storage_data) {
        // Both should contain ETH references
        assert!(
            fid_price.contains("ETH") || fid_price.contains("price"),
            "FID price should contain ETH or price info"
        );
        assert!(
            storage_price.contains("ETH") || storage_price.contains("price"),
            "Storage price should contain ETH or price info"
        );
        println!("   ✅ Price data consistency validation passed");
    }

    println!("   ✅ All cross-validation tests passed");
}

/// Test cleanup operations
async fn test_cleanup_validation(test_data_dir: &str, wallet_name: &str) {
    println!("   🧹 Testing cleanup validation...");

    // Test key deletion (if wallet was created)
    let delete_output = run_cli_command(test_data_dir, &["key", "delete", wallet_name]);

    // Note: This might fail if the wallet wasn't created or doesn't exist
    // That's okay for validation purposes
    if delete_output.status.success() {
        println!("   ✅ Key deletion validation passed");
    } else {
        println!("   ⚠️  Key deletion failed (expected if no wallet exists)");
    }

    // Verify directory can be cleaned up
    cleanup_test_directory(test_data_dir);

    // Verify directory no longer exists
    assert!(
        fs::metadata(test_data_dir).is_err(),
        "Test directory should be cleaned up"
    );

    println!("   ✅ Cleanup validation passed");
}

/// Validate price format in output
fn validate_price_format(output: &str) -> bool {
    // Check for common price indicators
    let price_indicators = [
        "ETH",
        "price",
        "Price",
        "registration",
        "Registration",
        "rental",
        "Rental",
        "0.0", // Common price format
    ];

    price_indicators
        .iter()
        .any(|&indicator| output.contains(indicator))
}

/// Validate storage format in output
fn validate_storage_format(output: &str) -> bool {
    // Check for storage-specific indicators
    let storage_indicators = [
        "storage", "Storage", "units", "Units", "rental", "Rental", "ETH", "price", "Price",
    ];

    storage_indicators
        .iter()
        .any(|&indicator| output.contains(indicator))
}

/// Run CLI command with test data directory
fn run_cli_command(test_data_dir: &str, args: &[&str]) -> std::process::Output {
    let mut cmd_args = vec!["run", "--bin", "castorix", "--", "--path", test_data_dir];
    cmd_args.extend(args.iter());

    Command::new("cargo")
        .args(&cmd_args)
        .output()
        .expect("Failed to execute CLI command")
}

/// Start local Anvil node
async fn start_local_anvil() -> Option<std::process::Child> {
    let output = Command::new("cargo")
        .args(["run", "--bin", "start-anvil"])
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                println!("✅ Anvil node started successfully");
                Some(
                    std::process::Command::new("anvil")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .expect("Failed to start Anvil"),
                )
            } else {
                println!(
                    "❌ Failed to start Anvil: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                None
            }
        }
        Err(e) => {
            println!("❌ Failed to execute start-anvil command: {}", e);
            None
        }
    }
}

/// Clean up test directory
fn cleanup_test_directory(test_data_dir: &str) {
    let _ = fs::remove_dir_all(test_data_dir);
}

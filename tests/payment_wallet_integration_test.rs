use std::process::Command;
use std::str::FromStr;

use anyhow::Result;
use castorix::farcaster::contracts::types::ContractAddresses;
use castorix::farcaster::contracts::FarcasterContractClient;
use ethers::signers::LocalWallet;
use ethers::signers::Signer;
use rand::rngs::OsRng;

/// Integration test for separate payment wallet functionality
#[tokio::test]
async fn test_payment_wallet_cli_integration() -> Result<()> {
    // Skip test if not in test environment
    if std::env::var("RUNNING_TESTS").is_err() {
        println!("⏭️  Skipping test (not in test environment)");
        return Ok(());
    }

    println!("🔗 Testing payment wallet CLI integration...");

    // Setup test environment
    let test_dir = "./test_payment_wallet_integration";

    // Clean up any existing test directory
    let _ = std::fs::remove_dir_all(test_dir);
    std::fs::create_dir_all(test_dir)?;

    // Generate test wallets
    let custody_wallet = LocalWallet::new(&mut OsRng);
    let payment_wallet = LocalWallet::new(&mut OsRng);

    println!("   Custody wallet: {}", custody_wallet.address());
    println!("   Payment wallet: {}", payment_wallet.address());

    // Test 1: Generate encrypted custody wallet
    println!("🔐 Testing custody wallet generation...");
    let custody_private_key = format!("{:x}", custody_wallet.signer().to_bytes());

    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd
        .args([
            "--path",
            test_dir,
            "key",
            "generate-encrypted",
            "--wallet",
            "custody_wallet",
        ])
        .env("PRIVATE_KEY", &custody_private_key)
        .output()?;

    if !output.status.success() {
        panic!(
            "❌ Custody wallet generation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!("✅ Custody wallet generated successfully");

    // Test 2: Generate encrypted payment wallet
    println!("💳 Testing payment wallet generation...");
    let payment_private_key = format!("{:x}", payment_wallet.signer().to_bytes());

    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd
        .args([
            "--path",
            test_dir,
            "key",
            "generate-encrypted",
            "--wallet",
            "payment_wallet",
        ])
        .env("PRIVATE_KEY", &payment_private_key)
        .output()?;

    if !output.status.success() {
        panic!(
            "❌ Payment wallet generation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    println!("✅ Payment wallet generated successfully");

    // Test 3: List wallets to verify both exist
    println!("📋 Testing wallet listing...");
    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd.args(["--path", test_dir, "key", "list"]).output()?;

    if !output.status.success() {
        panic!(
            "❌ Wallet listing failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        output_str.contains("custody_wallet"),
        "Custody wallet should be listed"
    );
    assert!(
        output_str.contains("payment_wallet"),
        "Payment wallet should be listed"
    );
    println!("✅ Both wallets listed successfully");

    // Test 4: Test storage price query with different wallets
    println!("💰 Testing storage price query...");
    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd
        .args([
            "--path", test_dir, "storage", "price", "999999", // Test FID
            "--units", "3",
        ])
        .output()?;

    if !output.status.success() {
        panic!(
            "❌ Storage price query failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        output_str.contains("Price per unit") || output_str.contains("Total price"),
        "Price information should be displayed"
    );
    println!("✅ Storage price query successful");

    // Test 5: Test storage rental with payment wallet (dry run)
    println!("🔄 Testing storage rental with payment wallet (dry run)...");
    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd
        .args([
            "--path",
            test_dir,
            "storage",
            "rent",
            "999999", // Test FID
            "--units",
            "1",
            "--payment-wallet",
            "payment_wallet",
            "--dry-run",
        ])
        .output()?;

    if !output.status.success() {
        panic!(
            "❌ Storage rental dry run failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        output_str.contains("payment wallet") || output_str.contains("Payment wallet"),
        "Payment wallet should be mentioned in output"
    );
    println!("✅ Storage rental dry run with payment wallet successful");

    // Clean up test directory
    let _ = std::fs::remove_dir_all(test_dir);

    println!("✅ Payment wallet CLI integration tests passed!");
    Ok(())
}

/// Test payment wallet error scenarios
#[tokio::test]
async fn test_payment_wallet_error_scenarios() -> Result<()> {
    // Skip test if not in test environment
    if std::env::var("RUNNING_TESTS").is_err() {
        println!("⏭️  Skipping test (not in test environment)");
        return Ok(());
    }

    println!("⚠️  Testing payment wallet error scenarios...");

    // Setup test environment
    let test_dir = "./test_payment_wallet_errors";

    // Clean up any existing test directory
    let _ = std::fs::remove_dir_all(test_dir);
    std::fs::create_dir_all(test_dir)?;

    // Generate test wallet
    let custody_wallet = LocalWallet::new(&mut OsRng);
    let custody_private_key = format!("{:x}", custody_wallet.signer().to_bytes());

    // Create only custody wallet
    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd
        .args([
            "--path",
            test_dir,
            "key",
            "generate-encrypted",
            "--wallet",
            "custody_wallet",
        ])
        .env("PRIVATE_KEY", &custody_private_key)
        .output()?;

    if !output.status.success() {
        panic!(
            "❌ Custody wallet generation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Test 1: Try to use non-existent payment wallet
    println!("🔍 Testing non-existent payment wallet error...");
    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd
        .args([
            "--path",
            test_dir,
            "storage",
            "rent",
            "999999",
            "--units",
            "1",
            "--payment-wallet",
            "non_existent_wallet",
            "--dry-run",
        ])
        .output()?;

    // This should fail because the payment wallet doesn't exist
    if output.status.success() {
        panic!("❌ Expected error for non-existent payment wallet, but command succeeded");
    }

    let error_output = String::from_utf8_lossy(&output.stderr);
    assert!(
        error_output.contains("not found")
            || error_output.contains("error")
            || error_output.contains("failed"),
        "Should show error for non-existent payment wallet"
    );
    println!("✅ Non-existent payment wallet correctly rejected");

    // Test 2: Try to use same wallet for both custody and payment
    println!("🔍 Testing same wallet for custody and payment...");
    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd
        .args([
            "--path",
            test_dir,
            "storage",
            "rent",
            "999999",
            "--units",
            "1",
            "--payment-wallet",
            "custody_wallet",
            "--dry-run",
        ])
        .output()?;

    // This should succeed (same wallet for both)
    if !output.status.success() {
        panic!(
            "❌ Same wallet for custody and payment should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    assert!(
        output_str.contains("custody wallet") || output_str.contains("same"),
        "Should indicate using same wallet for both"
    );
    println!("✅ Same wallet for custody and payment handled correctly");

    // Clean up test directory
    let _ = std::fs::remove_dir_all(test_dir);

    println!("✅ Payment wallet error scenario tests passed!");
    Ok(())
}

/// Test payment wallet with different FID scenarios
#[tokio::test]
async fn test_payment_wallet_different_fid_scenarios() -> Result<()> {
    // Skip test if not in test environment
    if std::env::var("RUNNING_TESTS").is_err() {
        println!("⏭️  Skipping test (not in test environment)");
        return Ok(());
    }

    println!("🎯 Testing payment wallet with different FID scenarios...");

    // Setup test environment
    let test_dir = "./test_payment_wallet_fid";

    // Clean up any existing test directory
    let _ = std::fs::remove_dir_all(test_dir);
    std::fs::create_dir_all(test_dir)?;

    // Generate test wallets
    let custody_wallet = LocalWallet::new(&mut OsRng);
    let payment_wallet = LocalWallet::new(&mut OsRng);

    let custody_private_key = format!("{:x}", custody_wallet.signer().to_bytes());
    let payment_private_key = format!("{:x}", payment_wallet.signer().to_bytes());

    // Create both wallets
    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd
        .args([
            "--path",
            test_dir,
            "key",
            "generate-encrypted",
            "--wallet",
            "custody_wallet",
        ])
        .env("PRIVATE_KEY", &custody_private_key)
        .output()?;

    if !output.status.success() {
        panic!("❌ Custody wallet generation failed");
    }

    let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
    let output = cmd
        .args([
            "--path",
            test_dir,
            "key",
            "generate-encrypted",
            "--wallet",
            "payment_wallet",
        ])
        .env("PRIVATE_KEY", &payment_private_key)
        .output()?;

    if !output.status.success() {
        panic!("❌ Payment wallet generation failed");
    }

    // Test different FID values
    let test_fids = vec!["12345", "999999", "1000000"];

    for fid in test_fids {
        println!("🔍 Testing FID: {}", fid);

        // Test storage price query for this FID
        let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
        let output = cmd
            .args(["--path", test_dir, "storage", "price", fid, "--units", "1"])
            .output()?;

        if !output.status.success() {
            panic!(
                "❌ Storage price query failed for FID {}: {}",
                fid,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(
            output_str.contains("Price") || output_str.contains("price"),
            "Price information should be displayed for FID {}",
            fid
        );

        // Test storage rental dry run for this FID
        let mut cmd = Command::new("./target/aarch64-apple-darwin/debug/castorix");
        let output = cmd
            .args([
                "--path",
                test_dir,
                "storage",
                "rent",
                fid,
                "--units",
                "1",
                "--payment-wallet",
                "payment_wallet",
                "--dry-run",
            ])
            .output()?;

        if !output.status.success() {
            panic!(
                "❌ Storage rental dry run failed for FID {}: {}",
                fid,
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        assert!(
            output_str.contains("payment wallet") || output_str.contains("Payment wallet"),
            "Payment wallet should be mentioned for FID {}",
            fid
        );

        println!("✅ FID {} tests passed", fid);
    }

    // Clean up test directory
    let _ = std::fs::remove_dir_all(test_dir);

    println!("✅ Payment wallet FID scenario tests passed!");
    Ok(())
}

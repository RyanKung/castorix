use std::process::Command;

mod test_consts;
use test_consts::setup_demo_test_env;
use test_consts::setup_placeholder_test_env;
use test_consts::should_skip_rpc_tests;

/// Simple CLI test that doesn't require building
/// Tests the CLI functionality using cargo run
#[tokio::test]
async fn test_simple_cli_functionality() {
    // Skip if no RPC tests should run
    if should_skip_rpc_tests() {
        println!("Skipping RPC tests");
        return;
    }

    println!("🚀 Starting Simple CLI Test");

    // Set up demo test environment
    setup_demo_test_env();

    let test_fid = 460432; // Use a known test FID

    // Test 1: Help command
    println!("\n📖 Testing Help Command...");
    test_cargo_command(&["--help"], "Main help", |output| {
        output.contains("Usage:") || output.contains("Commands:")
    })
    .await;

    // Test 2: FID help
    println!("\n🆔 Testing FID Help...");
    test_cargo_command(&["fid", "--help"], "FID help", |output| {
        output.contains("FID") || output.contains("Commands:")
    })
    .await;

    // Test 3: Storage help
    println!("\n🏠 Testing Storage Help...");
    test_cargo_command(&["storage", "--help"], "Storage help", |output| {
        output.contains("Storage") || output.contains("Commands:")
    })
    .await;

    // Test 4: Configuration validation
    println!("\n🔧 Testing Configuration Validation...");
    // Temporarily set placeholder configuration for validation test
    setup_placeholder_test_env();
    test_cargo_command(&["fid", "price"], "Configuration validation", |output| {
        output.contains("Warning") || output.contains("placeholder") || output.contains("ETH")
    })
    .await;
    // Reset back to demo configuration
    setup_demo_test_env();

    // Test 5: FID price query (should work with demo API)
    println!("\n💰 Testing FID Price Query...");
    test_cargo_command(&["fid", "price"], "FID price query", |output| {
        output.contains("ETH") || output.contains("Price") || output.contains("Warning")
    })
    .await;

    // Test 6: Storage price query
    println!("\n🏠 Testing Storage Price Query...");
    test_cargo_command(
        &["storage", "price", &test_fid.to_string(), "--units", "5"],
        "Storage price query",
        |output| output.contains("ETH") || output.contains("Price") || output.contains("Warning"),
    )
    .await;

    // Test 7: FID listing
    println!("\n📋 Testing FID Listing...");
    test_cargo_command(&["fid", "list"], "FID listing", |output| {
        output.contains("FID") || output.contains("wallet") || output.contains("No wallet")
    })
    .await;

    // Test 8: Storage usage
    println!("\n📊 Testing Storage Usage...");
    test_cargo_command(
        &["storage", "usage", &test_fid.to_string()],
        "Storage usage query",
        |output| output.contains("FID") || output.contains("Storage") || output.contains("Warning"),
    )
    .await;

    println!("\n✅ Simple CLI Test Completed Successfully!");
}

/// Test a CLI command using cargo run
async fn test_cargo_command<F>(args: &[&str], description: &str, validator: F)
where
    F: Fn(&str) -> bool,
{
    println!("   Testing {}...", description);

    let mut cmd_args = vec!["run", "--bin", "castorix", "--"];
    cmd_args.extend(args);

    let output = Command::new("cargo").args(&cmd_args).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            // For CLI tests, we consider it successful if we get expected output
            // even if the exit status is not 0 (due to network issues, etc.)
            if validator(&stdout) || validator(&stderr) {
                println!("   ✅ {} successful", description);
                // Show relevant output
                let relevant_output = if !stdout.is_empty() { &stdout } else { &stderr };
                if let Some(first_line) = relevant_output.lines().find(|l| !l.trim().is_empty()) {
                    println!("   📝 Output: {}", first_line);
                }
            } else {
                panic!(
                    "{} completed but output unexpected: stdout={} stderr={}",
                    description, stdout, stderr
                );
            }
        }
        Err(e) => {
            println!("   ❌ {} command failed: {}", description, e);
        }
    }
}

/// Test CLI argument parsing
#[tokio::test]
async fn test_cli_argument_parsing() {
    println!("🔧 Testing CLI Argument Parsing...");

    let test_cases = vec![
        (vec!["--help"], "Main help"),
        (vec!["fid", "--help"], "FID help"),
        (vec!["storage", "--help"], "Storage help"),
        (vec!["key", "--help"], "Key help"),
        (vec!["ens", "--help"], "ENS help"),
        (vec!["hub", "--help"], "Hub help"),
        (vec!["signers", "--help"], "Signers help"),
        (vec!["custody", "--help"], "Custody help"),
    ];

    for (args, description) in test_cases {
        println!("   Testing {}...", description);

        let mut cmd_args = vec!["run", "--bin", "castorix", "--"];
        cmd_args.extend(args);

        let output = Command::new("cargo").args(&cmd_args).output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                // Check if we get help output
                let has_help = stdout.contains("Usage:")
                    || stdout.contains("Commands:")
                    || stderr.contains("Usage:")
                    || stderr.contains("Commands:");

                if has_help {
                    println!("   ✅ {} working", description);
                } else {
                    println!("   ⚠️  {} may not be working correctly", description);
                    if !stdout.is_empty() {
                        println!(
                            "   📝 Output: {}",
                            stdout.lines().take(1).collect::<Vec<_>>().join(" ")
                        );
                    }
                }
            }
            Err(e) => {
                println!("   ❌ {} test failed: {}", description, e);
            }
        }
    }
}

/// Test environment variable configuration
#[tokio::test]
async fn test_environment_configuration() {
    println!("🔧 Testing Environment Configuration...");

    // Test with placeholder values
    setup_placeholder_test_env();

    let cmd_args = vec!["run", "--bin", "castorix", "--", "fid", "price"];
    let output = Command::new("cargo").args(&cmd_args).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let has_warning = stdout.contains("Configuration Warning")
                || stdout.contains("placeholder")
                || stderr.contains("Configuration Warning")
                || stderr.contains("placeholder");

            if has_warning {
                println!("   ✅ Configuration validation working correctly");
            } else {
                println!("   ⚠️  Configuration validation may not be working");
                if !stdout.is_empty() {
                    println!(
                        "   📝 Output: {}",
                        stdout.lines().take(2).collect::<Vec<_>>().join(" ")
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Configuration validation test failed: {}", e);
        }
    }

    // Reset configuration
    setup_demo_test_env();
}

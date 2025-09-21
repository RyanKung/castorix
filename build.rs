use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Check if we're running tests and need to start Anvil
    if env::var("CARGO_PKG_NAME").is_ok() && env::var("RUNNING_TESTS").is_ok() {
        setup_test_environment();
    }

    // Generate protobuf code from Snapchain's proto files
    let out_dir = "src/message";

    let proto_files = [
        "snapchain/src/proto/message.proto",
        "snapchain/src/proto/username_proof.proto",
    ];

    let mut codegen = protobuf_codegen_pure::Codegen::new();
    codegen.out_dir(out_dir);
    codegen.include("snapchain/src/proto");

    for proto_file in &proto_files {
        if Path::new(proto_file).exists() {
            codegen.input(proto_file);
        }
    }

    codegen.run().expect("protobuf codegen failed");

    // Compile Solidity contracts and generate ABIs
    compile_farcaster_contracts();

    // Generate Rust bindings from ABIs
    generate_rust_bindings();
}

fn compile_farcaster_contracts() {
    let contracts_dir = "contracts";
    let out_dir = "generated_abis";

    // Check if contracts submodule exists
    if !Path::new(contracts_dir).exists() {
        println!("cargo:warning=Contracts submodule not found, skipping ABI generation");
        return;
    }

    // Create output directory for ABIs
    if let Err(e) = fs::create_dir_all(out_dir) {
        println!("cargo:warning=Failed to create ABI output directory: {e}");
        return;
    }

    // Change to contracts directory and compile
    let original_dir = env::current_dir().expect("Failed to get current directory");

    if let Err(e) = env::set_current_dir(contracts_dir) {
        println!("cargo:warning=Failed to change to contracts directory: {e}");
        return;
    }

    // Install dependencies if needed
    let install_result = Command::new("forge").args(["install"]).output();

    match install_result {
        Ok(output) => {
            if !output.status.success() {
                println!(
                    "cargo:warning=Failed to install forge dependencies: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            println!("cargo:warning=Failed to run forge install: {e}");
        }
    }

    // Compile contracts
    let build_result = Command::new("forge")
        .args(["build", "--out", "../generated_abis"])
        .output();

    // Change back to original directory
    if let Err(e) = env::set_current_dir(original_dir) {
        println!("cargo:warning=Failed to change back to original directory: {e}");
    }

    match build_result {
        Ok(output) => {
            if !output.status.success() {
                println!(
                    "cargo:warning=Failed to compile contracts: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            } else {
                println!("cargo:info=Successfully compiled Farcaster contracts");
            }
        }
        Err(e) => {
            println!("cargo:warning=Failed to run forge build: {e}");
        }
    }

    // Tell Cargo to rerun this build script if contracts change
    println!("cargo:rerun-if-changed=contracts/src");
    println!("cargo:rerun-if-changed=contracts/foundry.toml");
}

fn generate_rust_bindings() {
    let abi_dir = "generated_abis";
    let bindings_dir = "src/farcaster/contracts/generated";

    // Check if ABI files exist
    if !Path::new(abi_dir).exists() {
        println!("cargo:warning=ABI directory not found, skipping Rust binding generation");
        return;
    }

    // Create bindings directory
    if let Err(e) = fs::create_dir_all(bindings_dir) {
        println!("cargo:warning=Failed to create bindings directory: {e}");
        return;
    }

    // Generate bindings for each contract
    let contracts = [
        ("IdRegistry", "IdRegistry.sol/IdRegistry.json"),
        ("KeyRegistry", "KeyRegistry.sol/KeyRegistry.json"),
        (
            "StorageRegistry",
            "StorageRegistry.sol/StorageRegistry.json",
        ),
        ("IdGateway", "IdGateway.sol/IdGateway.json"),
        ("KeyGateway", "KeyGateway.sol/KeyGateway.json"),
        ("Bundler", "Bundler.sol/Bundler.json"),
        (
            "SignedKeyRequestValidator",
            "SignedKeyRequestValidator.sol/SignedKeyRequestValidator.json",
        ),
        ("RecoveryProxy", "RecoveryProxy.sol/RecoveryProxy.json"),
    ];

    for (contract_name, abi_path) in &contracts {
        let abi_file = format!("{abi_dir}/{abi_path}");
        if Path::new(&abi_file).exists() {
            let binding_file = format!(
                "{}/{}_bindings.rs",
                bindings_dir,
                contract_name.to_lowercase()
            );
            println!("cargo:info=Generating bindings for {contract_name} from {abi_file}");

            // Use ethers abigen macro to generate bindings
            match generate_contract_bindings(contract_name, &abi_file, &binding_file) {
                Ok(_) => println!("cargo:info=Successfully generated bindings for {contract_name}"),
                Err(e) => {
                    println!("cargo:warning=Failed to generate bindings for {contract_name}: {e}")
                }
            }
        } else {
            println!("cargo:warning=ABI file not found: {abi_file}");
        }
    }

    // Create mod.rs file for the generated bindings (sorted alphabetically)
    let mut module_names: Vec<String> = contracts
        .iter()
        .map(|(name, _)| format!("pub mod {}_bindings;", name.to_lowercase()))
        .collect();
    module_names.sort(); // Sort alphabetically to ensure consistent formatting
    let mod_content = module_names.join("\n");

    let mod_file = format!("{bindings_dir}/mod.rs");
    if let Err(e) = fs::write(&mod_file, mod_content) {
        println!("cargo:warning=Failed to create mod.rs: {e}");
    }

    // Tell Cargo to rerun if ABI files change
    println!("cargo:rerun-if-changed={abi_dir}");
}

fn generate_contract_bindings(
    contract_name: &str,
    abi_file: &str,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read ABI content to validate file exists
    let _abi_content = fs::read_to_string(abi_file)?;

    // Generate the binding code using ethers abigen
    let binding_code = format!(
        r#"// Auto-generated bindings for {contract_name} contract
// Do not edit this file manually

use ethers::contract::abigen;

abigen!(
    {contract_name},
    "{abi_file}",
    event_derives(serde::Deserialize, serde::Serialize)
);
"#
    );

    // Write the binding file
    fs::write(output_file, binding_code)?;

    Ok(())
}

/// Setup test environment (start Anvil if needed)
fn setup_test_environment() {
    println!("cargo:warning=Setting up test environment...");

    // Check if Anvil is already running
    let anvil_running = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "http://localhost:8545",
        ])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() != "000")
        .unwrap_or(false);

    if !anvil_running {
        println!("cargo:warning=Starting Anvil for tests...");

        // Try to start Anvil in background
        let _ = Command::new("anvil")
            .args([
                "--host",
                "0.0.0.0",
                "--port",
                "8545",
                "--accounts",
                "10",
                "--balance",
                "10000",
            ])
            .spawn();

        // Wait a bit for Anvil to start
        std::thread::sleep(std::time::Duration::from_secs(2));
    } else {
        println!("cargo:warning=Anvil is already running on localhost:8545");
    }
}

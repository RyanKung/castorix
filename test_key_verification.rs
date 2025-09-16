use ethers::signers::{LocalWallet, Signer};

fn main() {
    // Test private key and address correspondence
    let test_keys = [
        ("0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"),
        ("0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d", "0x70997970c51812dc3a010c7d01b50e0d17dc79c8"),
        ("0x47e179ec197488593b187f80a5eb5f98febc56d7b53b03c5edf3c51a27d4e5f1", "0x15d34aaf54267db7d7c367839aaf71a00a2c6a65"),
    ];
    
    for (private_key, expected_address) in test_keys {
        match LocalWallet::from_str(private_key) {
            Ok(wallet) => {
                let address = wallet.address();
                println!("Private key: {}", private_key);
                println!("Generated address: {}", address);
                println!("Expected address: {}", expected_address);
                println!("Match: {}", address.to_string().to_lowercase() == expected_address.to_lowercase());
                println!("---");
            }
            Err(e) => {
                println!("Error parsing private key {}: {}", private_key, e);
            }
        }
    }
}

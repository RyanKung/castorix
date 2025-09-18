use castorix::farcaster::contracts::{types::ContractAddresses, FarcasterContractClient};
use ethers::{signers::LocalWallet, types::Address};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Farcaster Client Demo");

    // Create a read-only client
    let rpc_url = "http://127.0.0.1:8545".to_string();
    let addresses = ContractAddresses::default();

    let client = FarcasterContractClient::new(rpc_url, addresses.clone())?;

    println!("✅ Created Farcaster client");

    // Test network status
    println!("\n📊 Network Status:");
    let status = client.get_network_status().await?;
    println!("   Chain ID: {}", status.chain_id);
    println!("   Block Number: {}", status.block_number);
    println!("   ID Gateway Paused: {}", status.id_gateway_paused);
    println!("   Key Gateway Paused: {}", status.key_gateway_paused);
    println!(
        "   Storage Registry Paused: {}",
        status.storage_registry_paused
    );

    // Test FID information
    println!("\n🔍 FID Information:");
    let fid = 1u64;
    let fid_info = client.get_fid_info(fid).await?;
    println!("   FID: {}", fid_info.fid);
    println!("   Custody: {}", fid_info.custody);
    println!("   Recovery: {}", fid_info.recovery);
    println!("   Active Keys: {}", fid_info.active_keys);
    println!("   Inactive Keys: {}", fid_info.inactive_keys);
    println!("   Pending Keys: {}", fid_info.pending_keys);

    // Test address lookup
    println!("\n🏠 Address Lookup:");
    let test_address = Address::from_str("0x8773442740c17c9d0f0b87022c722f9a136206ed")?;
    if let Some(fid) = client.address_has_fid(test_address).await? {
        println!("   Address {} has FID: {}", test_address, fid);
    } else {
        println!("   Address {} does not have a FID", test_address);
    }

    // Test pricing
    println!("\n💰 Pricing Information:");
    let registration_price = client.get_registration_price().await?;
    println!(
        "   Registration Price: {} ETH",
        ethers::utils::format_ether(registration_price)
    );

    let storage_price = client.get_storage_price(1).await?;
    println!(
        "   Storage Price (1 unit): {} ETH",
        ethers::utils::format_ether(storage_price)
    );

    let storage_price_10 = client.get_storage_price(10).await?;
    println!(
        "   Storage Price (10 units): {} ETH",
        ethers::utils::format_ether(storage_price_10)
    );

    // Test wallet client (if we have a private key)
    println!("\n🔑 Wallet Client Demo:");
    let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    if let Ok(wallet) = LocalWallet::from_str(private_key) {
        let provider = ethers::providers::Provider::<ethers::providers::Http>::try_from(
            "http://127.0.0.1:8545",
        )?;
        let balance = provider.get_balance(wallet.address(), None).await?;
        println!("   Wallet Address: {}", wallet.address());
        println!(
            "   Wallet Balance: {} ETH",
            ethers::utils::format_ether(balance)
        );

        // Test if wallet has FID
        if let Some(fid) = client.address_has_fid(wallet.address()).await? {
            println!("   Wallet has FID: {}", fid);
        } else {
            println!("   Wallet does not have a FID");
        }
    } else {
        println!("   Could not create wallet client");
    }

    println!("\n🎉 Advanced Farcaster Client Demo Complete!");

    Ok(())
}
